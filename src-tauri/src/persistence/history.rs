use super::Database;
use crate::error::AppResult;
impl Database {
    /// Phase 3 only reads actual cooked events. Scheduling never updates this.
    pub(crate) async fn last_cooked(&self, meal_id: &str) -> AppResult<Option<String>> {
        Ok(
            sqlx::query_scalar("SELECT max(local_date) FROM meal_cooked_event WHERE meal_id=?")
                .bind(meal_id)
                .fetch_one(&self.pool)
                .await?,
        )
    }
}
