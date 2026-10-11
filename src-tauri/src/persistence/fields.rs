use super::{catalog::confirmation, Database};
use crate::{
    domain::{fields::*, validation},
    error::{AppError, AppResult},
};
use sqlx::{Row, SqliteConnection};
use uuid::Uuid;

impl Database {
    pub async fn all_fields(&self) -> AppResult<Vec<FieldDefinition>> {
        let mut connection = self.pool.acquire().await?;
        let ids: Vec<String> = sqlx::query_scalar(
            "SELECT id FROM custom_field_definition ORDER BY sort_order,normalized_label,id",
        )
        .fetch_all(&mut *connection)
        .await?;
        let mut result = vec![];
        for id in ids {
            result.push(load_field(&mut connection, &id).await?);
        }
        Ok(result)
    }
    pub async fn create_field(&self, input: FieldInput) -> AppResult<FieldDefinition> {
        input.validate()?;
        let id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO custom_field_definition(id,label,normalized_label,kind,description,options_json,minimum,maximum,unit_label,sort_order) VALUES (?,?,?,?,?,?,?,?,?,?)")
            .bind(&id).bind(input.label.trim()).bind(validation::normalized(&input.label)).bind(input.kind.as_str()).bind(&input.description).bind(serde_json::to_string(&input.options).map_err(|_|AppError::validation("Invalid field options."))?)
            .bind(input.minimum).bind(input.maximum).bind(&input.unit_label).bind(input.sort_order).execute(&self.pool).await?;
        self.field(&id).await
    }

    pub async fn field(&self, id: &str) -> AppResult<FieldDefinition> {
        validation::id(id)?;
        let mut connection = self.pool.acquire().await?;
        load_field(&mut connection, id).await
    }

    pub async fn list_fields(&self) -> AppResult<Vec<FieldDefinition>> {
        let mut connection = self.pool.acquire().await?;
        let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM custom_field_definition WHERE retired=0 ORDER BY sort_order,normalized_label,id").fetch_all(&mut *connection).await?;
        let mut fields = Vec::new();
        for id in ids {
            fields.push(load_field(&mut connection, &id).await?);
        }
        Ok(fields)
    }

    /// Editing the storage shape is deliberately unavailable. Rename labels,
    /// descriptions, order and existing options without deleting stored values.
    pub async fn update_field(&self, id: &str, input: FieldInput) -> AppResult<FieldDefinition> {
        validation::id(id)?;
        input.validate()?;
        let mut tx = self.pool.begin().await?;
        let old = load_field(&mut tx, id).await?;
        let old_ids: Vec<_> = old.data.options.iter().map(|o| &o.id).collect();
        let new_ids: Vec<_> = input.options.iter().map(|o| &o.id).collect();
        if old.retired
            || old.data.kind != input.kind
            || old.data.minimum != input.minimum
            || old.data.maximum != input.maximum
            || old.data.unit_label != input.unit_label
            || old_ids != new_ids
        {
            return Err(AppError::conflict());
        }
        sqlx::query("UPDATE custom_field_definition SET label=?,normalized_label=?,description=?,options_json=?,sort_order=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?")
            .bind(input.label.trim()).bind(validation::normalized(&input.label)).bind(&input.description).bind(serde_json::to_string(&input.options).map_err(|_|AppError::validation("Invalid field options."))?).bind(input.sort_order).bind(id).execute(&mut *tx).await?;
        let result = load_field(&mut tx, id).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn retire_field(&self, id: &str, confirmed: bool) -> AppResult<()> {
        validation::id(id)?;
        confirmation(confirmed)?;
        if sqlx::query("UPDATE custom_field_definition SET retired=1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?").bind(id).execute(&self.pool).await?.rows_affected()==0 { return Err(AppError::not_found()); }
        Ok(())
    }
}

pub(crate) async fn load_field(
    connection: &mut SqliteConnection,
    id: &str,
) -> AppResult<FieldDefinition> {
    let row = sqlx::query("SELECT * FROM custom_field_definition WHERE id=?")
        .bind(id)
        .fetch_one(connection)
        .await?;
    let kind: String = row.try_get("kind")?;
    let options: String = row.try_get("options_json")?;
    Ok(FieldDefinition {
        id: row.try_get("id")?,
        retired: row.try_get("retired")?,
        data: FieldInput {
            label: row.try_get("label")?,
            kind: serde_json::from_value(serde_json::Value::String(kind))
                .map_err(|_| AppError::storage())?,
            description: row.try_get("description")?,
            options: serde_json::from_str(&options).map_err(|_| AppError::storage())?,
            minimum: row.try_get("minimum")?,
            maximum: row.try_get("maximum")?,
            unit_label: row.try_get("unit_label")?,
            sort_order: row.try_get("sort_order")?,
        },
    })
}
