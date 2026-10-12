use super::Database;
use crate::{
    domain::{planning, shopping::Document, validation},
    error::{AppError, AppResult},
};
impl Database {
    pub(super) async fn validate_backup(&self) -> AppResult<()> {
        let ok: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;
        if ok != "ok"
            || !sqlx::query("PRAGMA foreign_key_check")
                .fetch_all(&self.pool)
                .await?
                .is_empty()
        {
            return Err(AppError::storage());
        }
        self.measurement_preferences().await?.validate()?;
        self.usability_preferences().await?;
        let fields = self.all_fields().await?;
        for field in &fields {
            validation::id(&field.id)?;
            field.data.validate()?;
        }
        let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM meal")
            .fetch_all(&self.pool)
            .await?;
        for id in ids {
            let meal = self.meal(&id).await?;
            meal.data.validate()?;
            for value in &meal.data.custom_fields {
                fields
                    .iter()
                    .find(|f| f.id == value.field_id)
                    .ok_or_else(AppError::storage)?
                    .data
                    .validate_value(&value.value)?;
            }
        }
        let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM ingredient")
            .fetch_all(&self.pool)
            .await?;
        for id in ids {
            self.ingredient(&id).await?.data.validate()?;
        }
        for (table, display, normalized) in [
            ("ingredient", "name", "normalized_name"),
            ("ingredient_alias", "alias", "normalized_alias"),
            ("ingredient_category", "name", "normalized_name"),
            ("shopping_section", "name", "normalized_name"),
            ("custom_field_definition", "label", "normalized_label"),
        ] {
            let pairs: Vec<(String, String)> =
                sqlx::query_as(&format!("SELECT {display},{normalized} FROM {table}"))
                    .fetch_all(&self.pool)
                    .await?;
            for (text, stored) in pairs {
                if validation::normalized(&text) != stored {
                    return Err(AppError::validation(
                        "Invalid normalized identity in backup.",
                    ));
                }
            }
        }
        let plans: Vec<(String, Option<String>, i64, i64)> = sqlx::query_as(
            "SELECT week_start,local_date,servings_numerator,servings_denominator FROM plan_entry",
        )
        .fetch_all(&self.pool)
        .await?;
        for (week, day, n, d) in plans {
            planning::validate_day(&week, &day)?;
            validation::Amount {
                numerator: n,
                denominator: d,
            }
            .validate(true)?;
        }
        let dates: Vec<String> = sqlx::query_scalar("SELECT local_date FROM meal_cooked_event")
            .fetch_all(&self.pool)
            .await?;
        for date in dates {
            planning::local_date(&date)?;
        }
        let docs: Vec<String> = sqlx::query_scalar(
            "SELECT document_json FROM shopping_list WHERE document_json IS NOT NULL",
        )
        .fetch_all(&self.pool)
        .await?;
        for text in docs {
            let doc: Document = serde_json::from_str(&text).map_err(|_| AppError::storage())?;
            if doc.version != 1 || doc.items.len() > 10_000 {
                return Err(AppError::storage());
            }
            let mut ids = std::collections::HashSet::new();
            for item in doc.items {
                let units = [
                    (item.unit_id.clone(), false),
                    (
                        item.adjustment.as_ref().and_then(|e| e.unit_id.clone()),
                        true,
                    ),
                ];
                for (id, adjustment) in units {
                    let Some(id) = id else {
                        continue;
                    };
                    validation::id(&id)?;
                    let valid: i64 = sqlx::query_scalar(
                        "SELECT count(*) FROM unit WHERE id=? AND (?=0 OR dimension!='temperature')",
                    )
                    .bind(id)
                    .bind(adjustment)
                    .fetch_one(&self.pool)
                    .await?;
                    if valid != 1 {
                        return Err(AppError::storage());
                    }
                }
                validation::text(&item.id, 1000, true)?;
                if !ids.insert(item.id) {
                    return Err(AppError::storage());
                }
                match (item.numerator, item.denominator) {
                    (Some(n), Some(d)) => {
                        let n = n.parse::<i128>().map_err(|_| AppError::storage())?;
                        let d = d.parse::<i128>().map_err(|_| AppError::storage())?;
                        if n < 0 {
                            return Err(AppError::storage());
                        }
                        crate::domain::measurements::rational::Rational::new(n, d)?;
                    }
                    (None, None) => (),
                    _ => return Err(AppError::storage()),
                }
                if let Some(edit) = item.adjustment {
                    validation::text(&edit.amount, 120, false)?;
                    if crate::domain::measurements::rational::parse(&edit.amount)?
                        .is_some_and(|q| q.n < 0)
                    {
                        return Err(AppError::storage());
                    }
                    validation::optional_id(&edit.unit_id)?;
                }
            }
        }
        Ok(())
    }
}
