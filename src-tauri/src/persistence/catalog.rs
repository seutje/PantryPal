use super::Database;
use crate::{
    domain::{catalog::*, validation},
    error::{AppError, AppResult},
};
use sqlx::{Row, Sqlite, SqliteConnection, Transaction};
use uuid::Uuid;

impl Database {
    pub async fn catalog(&self) -> AppResult<Catalog> {
        let mut tx = self.pool.begin().await?;
        let ids: Vec<String> = sqlx::query_scalar(
            "SELECT id FROM ingredient WHERE retired=0 ORDER BY normalized_name,id",
        )
        .fetch_all(&mut *tx)
        .await?;
        let mut ingredients = Vec::with_capacity(ids.len());
        for id in ids {
            ingredients.push(load_ingredient(&mut tx, &id).await?);
        }
        // Keep retired labels readable for existing references. Future pickers
        // filter `retired` before offering new assignments.
        let categories = sqlx::query_as("SELECT id,name,0 AS sort_order,retired FROM ingredient_category ORDER BY normalized_name,id").fetch_all(&mut *tx).await?;
        let sections = sqlx::query_as("SELECT id,name,sort_order,retired FROM shopping_section ORDER BY sort_order,normalized_name,id").fetch_all(&mut *tx).await?;
        let units = sqlx::query_as("SELECT * FROM unit ORDER BY dimension,system,id")
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(Catalog {
            ingredients,
            categories,
            sections,
            units,
        })
    }

    pub async fn ingredient(&self, id: &str) -> AppResult<Ingredient> {
        validation::id(id)?;
        let mut tx = self.pool.begin().await?;
        let result = load_ingredient(&mut tx, id).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn create_ingredient(&self, input: IngredientInput) -> AppResult<Ingredient> {
        input.validate()?;
        let id = Uuid::new_v4().to_string();
        let mut tx = self.pool.begin().await?;
        validate_references(&mut tx, &input, None).await?;
        sqlx::query("INSERT INTO ingredient(id,name,normalized_name,category_id,section_id,preferred_unit_id,notes) VALUES (?,?,?,?,?,?,?)")
            .bind(&id).bind(input.name.trim()).bind(validation::normalized(&input.name)).bind(&input.category_id).bind(&input.section_id).bind(&input.preferred_unit_id).bind(&input.notes)
            .execute(&mut *tx).await?;
        write_children(&mut tx, &id, &input).await?;
        let result = load_ingredient(&mut tx, &id).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn update_ingredient(
        &self,
        id: &str,
        input: IngredientInput,
    ) -> AppResult<Ingredient> {
        validation::id(id)?;
        input.validate()?;
        let mut tx = self.pool.begin().await?;
        let old = load_ingredient(&mut tx, id).await?;
        if old.retired {
            return Err(AppError::conflict());
        }
        validate_references(&mut tx, &input, Some(&old.data)).await?;
        // Replacing child rows is part of the same explicit ingredient edit.
        sqlx::query("DELETE FROM ingredient_alias WHERE ingredient_id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM ingredient_season_month WHERE ingredient_id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE ingredient SET name=?,normalized_name=?,category_id=?,section_id=?,preferred_unit_id=?,notes=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?")
            .bind(input.name.trim()).bind(validation::normalized(&input.name)).bind(&input.category_id).bind(&input.section_id).bind(&input.preferred_unit_id).bind(&input.notes).bind(id)
            .execute(&mut *tx).await?;
        write_children(&mut tx, id, &input).await?;
        let result = load_ingredient(&mut tx, id).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn retire_ingredient(&self, id: &str, confirmed: bool) -> AppResult<()> {
        validation::id(id)?;
        confirmation(confirmed)?;
        let changed = sqlx::query("UPDATE ingredient SET retired=1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?").bind(id).execute(&self.pool).await?;
        if changed.rows_affected() == 0 {
            return Err(AppError::not_found());
        }
        Ok(())
    }

    /// Vocabulary writes use an enum-selected static query, never a UI table name.
    pub async fn create_vocabulary(
        &self,
        kind: VocabularyKind,
        name: &str,
        sort_order: i64,
    ) -> AppResult<Vocabulary> {
        validation::text(name, 120, true)?;
        if sort_order < 0 {
            return Err(AppError::validation("Invalid section order."));
        }
        let id = Uuid::new_v4().to_string();
        match kind {
            VocabularyKind::Category => {
                sqlx::query(
                    "INSERT INTO ingredient_category(id,name,normalized_name) VALUES (?,?,?)",
                )
                .bind(&id)
                .bind(name.trim())
                .bind(validation::normalized(name))
                .execute(&self.pool)
                .await?;
            }
            VocabularyKind::Section => {
                sqlx::query("INSERT INTO shopping_section(id,name,normalized_name,sort_order) VALUES (?,?,?,?)").bind(&id).bind(name.trim()).bind(validation::normalized(name)).bind(sort_order).execute(&self.pool).await?;
            }
        }
        self.vocabulary(kind, &id).await
    }

    pub async fn vocabulary(&self, kind: VocabularyKind, id: &str) -> AppResult<Vocabulary> {
        validation::id(id)?;
        let query = match kind {
            VocabularyKind::Category => {
                "SELECT id,name,0 AS sort_order,retired FROM ingredient_category WHERE id=?"
            }
            VocabularyKind::Section => {
                "SELECT id,name,sort_order,retired FROM shopping_section WHERE id=?"
            }
        };
        Ok(sqlx::query_as(query).bind(id).fetch_one(&self.pool).await?)
    }

    pub async fn update_vocabulary(
        &self,
        kind: VocabularyKind,
        id: &str,
        name: &str,
        sort_order: i64,
    ) -> AppResult<Vocabulary> {
        validation::id(id)?;
        validation::text(name, 120, true)?;
        if sort_order < 0 {
            return Err(AppError::validation("Invalid section order."));
        }
        let changed = match kind {
            VocabularyKind::Category => sqlx::query("UPDATE ingredient_category SET name=?,normalized_name=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=? AND retired=0").bind(name.trim()).bind(validation::normalized(name)).bind(id).execute(&self.pool).await?,
            VocabularyKind::Section => sqlx::query("UPDATE shopping_section SET name=?,normalized_name=?,sort_order=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=? AND retired=0").bind(name.trim()).bind(validation::normalized(name)).bind(sort_order).bind(id).execute(&self.pool).await?,
        };
        if changed.rows_affected() == 0 {
            return Err(AppError::not_found());
        }
        self.vocabulary(kind, id).await
    }

    pub async fn retire_vocabulary(
        &self,
        kind: VocabularyKind,
        id: &str,
        confirmed: bool,
    ) -> AppResult<()> {
        validation::id(id)?;
        confirmation(confirmed)?;
        let query = match kind { VocabularyKind::Category=>"UPDATE ingredient_category SET retired=1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?", VocabularyKind::Section=>"UPDATE shopping_section SET retired=1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?" };
        if sqlx::query(query)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected()
            == 0
        {
            return Err(AppError::not_found());
        }
        Ok(())
    }
}

pub(crate) fn confirmation(confirmed: bool) -> AppResult<()> {
    if !confirmed {
        return Err(AppError::validation(
            "Confirm retirement before hiding this record.",
        ));
    }
    Ok(())
}

async fn validate_references(
    tx: &mut Transaction<'_, Sqlite>,
    input: &IngredientInput,
    old: Option<&IngredientInput>,
) -> AppResult<()> {
    if let Some(id) = &input.category_id {
        let query = if old.is_some_and(|old| old.category_id.as_ref() == Some(id)) {
            "SELECT count(*) FROM ingredient_category WHERE id=?"
        } else {
            "SELECT count(*) FROM ingredient_category WHERE id=? AND retired=0"
        };
        ensure_reference(tx, query, id).await?;
    }
    if let Some(id) = &input.section_id {
        let query = if old.is_some_and(|old| old.section_id.as_ref() == Some(id)) {
            "SELECT count(*) FROM shopping_section WHERE id=?"
        } else {
            "SELECT count(*) FROM shopping_section WHERE id=? AND retired=0"
        };
        ensure_reference(tx, query, id).await?;
    }
    if let Some(id) = &input.preferred_unit_id {
        ensure_reference(tx, "SELECT count(*) FROM unit WHERE id=?", id).await?;
    }
    Ok(())
}

pub(crate) async fn ensure_reference(
    connection: &mut SqliteConnection,
    query: &'static str,
    id: &str,
) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(query)
        .bind(id)
        .fetch_one(connection)
        .await?;
    if exists != 1 {
        return Err(AppError::conflict());
    }
    Ok(())
}

async fn write_children(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    input: &IngredientInput,
) -> AppResult<()> {
    for alias in &input.aliases {
        sqlx::query("INSERT INTO ingredient_alias(id,ingredient_id,alias,normalized_alias) VALUES (?,?,?,?)")
            .bind(Uuid::new_v4().to_string()).bind(id).bind(alias.trim()).bind(validation::normalized(alias)).execute(&mut **tx).await?;
    }
    for month in &input.season_months {
        sqlx::query("INSERT INTO ingredient_season_month(ingredient_id,month) VALUES (?,?)")
            .bind(id)
            .bind(i64::from(*month))
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn load_ingredient(tx: &mut Transaction<'_, Sqlite>, id: &str) -> AppResult<Ingredient> {
    let row = sqlx::query("SELECT * FROM ingredient WHERE id=?")
        .bind(id)
        .fetch_one(&mut **tx)
        .await?;
    let aliases = sqlx::query_scalar(
        "SELECT alias FROM ingredient_alias WHERE ingredient_id=? ORDER BY normalized_alias",
    )
    .bind(id)
    .fetch_all(&mut **tx)
    .await?;
    let months: Vec<i64> = sqlx::query_scalar(
        "SELECT month FROM ingredient_season_month WHERE ingredient_id=? ORDER BY month",
    )
    .bind(id)
    .fetch_all(&mut **tx)
    .await?;
    Ok(Ingredient {
        id: row.try_get("id")?,
        retired: row.try_get("retired")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        data: IngredientInput {
            name: row.try_get("name")?,
            category_id: row.try_get("category_id")?,
            section_id: row.try_get("section_id")?,
            preferred_unit_id: row.try_get("preferred_unit_id")?,
            notes: row.try_get("notes")?,
            aliases,
            season_months: months.into_iter().map(|month| month as u8).collect(),
        },
    })
}
