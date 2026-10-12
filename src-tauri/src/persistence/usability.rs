use super::Database;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsabilityPreferences {
    pub clipboard_url_prefill: bool,
    pub section_order: Vec<String>,
}
impl Database {
    pub async fn usability_preferences(&self) -> AppResult<UsabilityPreferences> {
        let raw: Option<String> = sqlx::query_scalar(
            "SELECT value_json FROM app_settings WHERE key='import.clipboard_prefill'",
        )
        .fetch_optional(&self.pool)
        .await?;
        let clipboard_url_prefill = raw
            .map(|s| serde_json::from_str(&s).map_err(|_| AppError::storage()))
            .transpose()?
            .unwrap_or(false);
        let section_order = sqlx::query_scalar(
            "SELECT id FROM shopping_section WHERE retired=0 ORDER BY sort_order,name,id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(UsabilityPreferences {
            clipboard_url_prefill,
            section_order,
        })
    }
    pub async fn save_usability_preferences(&self, input: UsabilityPreferences) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        let ids: Vec<String> =
            sqlx::query_scalar("SELECT id FROM shopping_section WHERE retired=0")
                .fetch_all(&mut *tx)
                .await?;
        let requested: std::collections::HashSet<_> = input.section_order.iter().collect();
        if ids.len() != input.section_order.len()
            || requested.len() != ids.len()
            || ids.iter().any(|i| !requested.contains(i))
        {
            return Err(AppError::validation(
                "Section order changed. Reload Settings before saving.",
            ));
        }
        for (index, id) in input.section_order.iter().enumerate() {
            sqlx::query("UPDATE shopping_section SET sort_order=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?").bind(index as i64).bind(id).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO app_settings(key,value_json) VALUES ('import.clipboard_prefill',?) ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')")
            .bind(if input.clipboard_url_prefill { "true" } else { "false" }).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
}
