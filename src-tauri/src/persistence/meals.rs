use super::{
    catalog::{confirmation, ensure_reference},
    fields::load_field,
    Database,
};
use crate::{
    domain::{
        meals::*,
        validation::{self, Amount},
    },
    error::{AppError, AppResult},
};
use sqlx::{Row, Sqlite, Transaction};
use uuid::Uuid;

impl Database {
    pub async fn create_meal(&self, input: MealInput) -> AppResult<Meal> {
        input.validate()?;
        let mut tx = self.pool.begin().await?;
        let result = insert_meal(&mut tx, &input).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn meal(&self, id: &str) -> AppResult<Meal> {
        validation::id(id)?;
        let mut tx = self.pool.begin().await?;
        let result = load_meal(&mut tx, id).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn list_meals(&self) -> AppResult<Vec<Meal>> {
        let mut tx = self.pool.begin().await?;
        let ids: Vec<String> =
            sqlx::query_scalar("SELECT id FROM meal WHERE retired=0 ORDER BY normalized_title,id")
                .fetch_all(&mut *tx)
                .await?;
        let mut meals = Vec::new();
        for id in ids {
            meals.push(load_meal(&mut tx, &id).await?);
        }
        tx.commit().await?;
        Ok(meals)
    }

    pub async fn update_meal(
        &self,
        id: &str,
        expected_revision: i64,
        mut input: MealInput,
    ) -> AppResult<Meal> {
        validation::id(id)?;
        input.validate()?;
        let mut tx = self.pool.begin().await?;
        let old = load_meal(&mut tx, id).await?;
        if old.retired || old.revision != expected_revision {
            return Err(AppError::conflict());
        }
        // Missing custom fields are retained (including retired definitions).
        // A caller supplies an explicit null value to change a value to unknown.
        for old_value in &old.data.custom_fields {
            if !input
                .custom_fields
                .iter()
                .any(|value| value.field_id == old_value.field_id)
            {
                input.custom_fields.push(old_value.clone());
            }
        }
        input.validate()?;
        sqlx::query("UPDATE meal SET title=?,normalized_title=?,description=?,servings_numerator=?,servings_denominator=?,prep_minutes=?,cook_minutes=?,instructions=?,notes=?,source_url=?,source_site=?,base_servings_text=?,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?")
            .bind(input.title.trim()).bind(validation::normalized(&input.title)).bind(&input.description).bind(input.base_servings.numerator).bind(input.base_servings.denominator).bind(input.prep_minutes).bind(input.cook_minutes).bind(&input.instructions).bind(&input.notes).bind(&input.source_url).bind(&input.source_site).bind(&input.base_servings_text).bind(id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM meal_ingredient WHERE meal_id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM meal_tag WHERE meal_id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM meal_custom_field_value WHERE meal_id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        write_children(&mut tx, id, &input, Some(&old)).await?;
        let result = load_meal(&mut tx, id).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn retire_meal(&self, id: &str, confirmed: bool) -> AppResult<()> {
        validation::id(id)?;
        confirmation(confirmed)?;
        if sqlx::query("UPDATE meal SET retired=1,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=? AND retired=0").bind(id).execute(&self.pool).await?.rows_affected()==0 {
            // Retirement is idempotent, but an unknown ID is still an error.
            self.meal(id).await?;
        }
        Ok(())
    }
}

async fn write_children(
    tx: &mut Transaction<'_, Sqlite>,
    meal_id: &str,
    input: &MealInput,
    old: Option<&Meal>,
) -> AppResult<()> {
    for (index, line) in input.ingredients.iter().enumerate() {
        if let Some(unit_id) = &line.unit_id {
            let previous = old.is_some_and(|meal| {
                meal.data
                    .ingredients
                    .iter()
                    .any(|prior| prior.id == line.id && prior.unit_id.as_ref() == Some(unit_id))
            });
            ensure_reference(
                tx,
                if previous {
                    "SELECT count(*) FROM unit WHERE id=?"
                } else {
                    "SELECT count(*) FROM unit WHERE id=? AND dimension<>'temperature'"
                },
                unit_id,
            )
            .await?;
        }
        if let Some(ingredient_id) = &line.ingredient_id {
            let previously_used = old.is_some_and(|old| {
                old.data
                    .ingredients
                    .iter()
                    .any(|old_line| old_line.ingredient_id.as_ref() == Some(ingredient_id))
            });
            let query = if previously_used {
                "SELECT count(*) FROM ingredient WHERE id=?"
            } else {
                "SELECT count(*) FROM ingredient WHERE id=? AND retired=0"
            };
            ensure_reference(tx, query, ingredient_id).await?;
        }
        sqlx::query("INSERT INTO meal_ingredient(id,meal_id,ingredient_id,quantity_numerator,quantity_denominator,unit_id,raw_text,source_convention,note,optional,sort_order,amount_text) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(&line.id).bind(meal_id).bind(&line.ingredient_id).bind(line.quantity.as_ref().map(|q|q.numerator)).bind(line.quantity.as_ref().map(|q|q.denominator)).bind(&line.unit_id).bind(&line.raw_text).bind(&line.source_convention).bind(&line.note).bind(line.optional).bind(index as i64).bind(&line.amount_text).execute(&mut **tx).await?;
    }
    for tag in &input.tags {
        sqlx::query("INSERT INTO meal_tag(meal_id,tag) VALUES (?,?)")
            .bind(meal_id)
            .bind(tag.trim())
            .execute(&mut **tx)
            .await?;
    }
    for value in &input.custom_fields {
        let definition = load_field(tx, &value.field_id).await?;
        if definition.retired
            && !old.is_some_and(|old| {
                old.data
                    .custom_fields
                    .iter()
                    .any(|old_value| old_value == value)
            })
        {
            return Err(AppError::conflict());
        }
        definition.data.validate_value(&value.value)?;
        let payload = value
            .value
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|_| AppError::validation("Invalid custom field value."))?;
        sqlx::query(
            "INSERT INTO meal_custom_field_value(meal_id,field_id,value_json) VALUES (?,?,?)",
        )
        .bind(meal_id)
        .bind(&value.field_id)
        .bind(payload)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub(crate) async fn load_meal(tx: &mut Transaction<'_, Sqlite>, id: &str) -> AppResult<Meal> {
    let row = sqlx::query("SELECT * FROM meal WHERE id=?")
        .bind(id)
        .fetch_one(&mut **tx)
        .await?;
    let lines = sqlx::query("SELECT * FROM meal_ingredient WHERE meal_id=? ORDER BY sort_order")
        .bind(id)
        .fetch_all(&mut **tx)
        .await?;
    let mut ingredients = Vec::new();
    for line in lines {
        let numerator: Option<i64> = line.try_get("quantity_numerator")?;
        let denominator: Option<i64> = line.try_get("quantity_denominator")?;
        ingredients.push(MealIngredient {
            id: line.try_get("id")?,
            ingredient_id: line.try_get("ingredient_id")?,
            quantity: match (numerator, denominator) {
                (Some(numerator), Some(denominator)) => Some(Amount {
                    numerator,
                    denominator,
                }),
                (None, None) => None,
                _ => return Err(AppError::storage()),
            },
            amount_text: line.try_get("amount_text")?,
            unit_id: line.try_get("unit_id")?,
            raw_text: line.try_get("raw_text")?,
            source_convention: line.try_get("source_convention")?,
            note: line.try_get("note")?,
            optional: line.try_get("optional")?,
        });
    }
    let tags = sqlx::query_scalar("SELECT tag FROM meal_tag WHERE meal_id=? ORDER BY tag")
        .bind(id)
        .fetch_all(&mut **tx)
        .await?;
    let values = sqlx::query(
        "SELECT field_id,value_json FROM meal_custom_field_value WHERE meal_id=? ORDER BY field_id",
    )
    .bind(id)
    .fetch_all(&mut **tx)
    .await?;
    let mut custom_fields = Vec::new();
    for value in values {
        let payload: Option<String> = value.try_get("value_json")?;
        custom_fields.push(MealFieldValue {
            field_id: value.try_get("field_id")?,
            value: payload
                .map(|value| serde_json::from_str(&value))
                .transpose()
                .map_err(|_| AppError::storage())?,
        });
    }
    Ok(Meal {
        id: row.try_get("id")?,
        retired: row.try_get("retired")?,
        revision: row.try_get("revision")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        data: MealInput {
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            base_servings: Amount {
                numerator: row.try_get("servings_numerator")?,
                denominator: row.try_get("servings_denominator")?,
            },
            base_servings_text: row.try_get("base_servings_text")?,
            prep_minutes: row.try_get("prep_minutes")?,
            cook_minutes: row.try_get("cook_minutes")?,
            instructions: row.try_get("instructions")?,
            notes: row.try_get("notes")?,
            source_url: row.try_get("source_url")?,
            source_site: row.try_get("source_site")?,
            tags,
            ingredients,
            custom_fields,
        },
    })
}

pub(crate) async fn insert_meal(
    tx: &mut Transaction<'_, Sqlite>,
    input: &MealInput,
) -> AppResult<Meal> {
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO meal(id,title,normalized_title,description,servings_numerator,servings_denominator,prep_minutes,cook_minutes,instructions,notes,source_url,source_site,base_servings_text) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(&id).bind(input.title.trim()).bind(validation::normalized(&input.title)).bind(&input.description).bind(input.base_servings.numerator).bind(input.base_servings.denominator).bind(input.prep_minutes).bind(input.cook_minutes).bind(&input.instructions).bind(&input.notes).bind(&input.source_url).bind(&input.source_site).bind(&input.base_servings_text).execute(&mut **tx).await?;
    write_children(tx, &id, input, None).await?;
    let result = load_meal(tx, &id).await?;
    Ok(result)
}
