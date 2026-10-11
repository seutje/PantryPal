use super::Database;
use super::{
    catalog::confirmation,
    planning::{bump_week, receipt, replay},
};
use crate::{
    domain::{planning::*, validation},
    error::{AppError, AppResult},
};
use sqlx::{Row, SqliteConnection};
use uuid::Uuid;
pub(crate) async fn validate_cooked_date(
    connection: &mut SqliteConnection,
    date: &str,
) -> AppResult<()> {
    local_date(date)?;
    let today: String = sqlx::query_scalar("SELECT date('now','localtime')")
        .fetch_one(connection)
        .await?;
    if date > today.as_str() {
        return Err(AppError::validation(
            "Cooking history cannot be recorded in the future.",
        ));
    }
    Ok(())
}
pub(crate) async fn load_event(
    connection: &mut SqliteConnection,
    id: &str,
) -> AppResult<CookedEvent> {
    let row = sqlx::query("SELECT * FROM meal_cooked_event WHERE id=?")
        .bind(id)
        .fetch_one(connection)
        .await?;
    Ok(CookedEvent {
        id: row.try_get("id")?,
        meal_id: row.try_get("meal_id")?,
        local_date: row.try_get("local_date")?,
        plan_entry_id: row.try_get("plan_entry_id")?,
        origin_plan_entry_id: row.try_get("origin_plan_entry_id")?,
        voided: row.try_get("voided")?,
        revision: row.try_get("revision")?,
    })
}
pub(crate) async fn change_event(
    connection: &mut SqliteConnection,
    event: &CookedEvent,
    date: &str,
    voided: bool,
) -> AppResult<()> {
    sqlx::query("INSERT INTO cooked_event_change(id,event_id,previous_date,next_date,previous_voided,next_voided) VALUES (?,?,?,?,?,?)").bind(Uuid::new_v4().to_string()).bind(&event.id).bind(&event.local_date).bind(date).bind(event.voided).bind(voided).execute(&mut *connection).await?;
    sqlx::query(
        "UPDATE meal_cooked_event SET local_date=?,voided=?,revision=revision+1 WHERE id=?",
    )
    .bind(date)
    .bind(voided)
    .bind(&event.id)
    .execute(connection)
    .await?;
    Ok(())
}
impl Database {
    /// Voided records remain stored, but never count as actual cooking.
    pub(crate) async fn last_cooked(&self, meal_id: &str) -> AppResult<Option<String>> {
        Ok(sqlx::query_scalar(
            "SELECT max(local_date) FROM meal_cooked_event WHERE meal_id=? AND voided=0",
        )
        .bind(meal_id)
        .fetch_one(&self.pool)
        .await?)
    }
    pub async fn meal_history(&self, meal_id: &str) -> AppResult<MealHistory> {
        validation::id(meal_id)?;
        self.meal(meal_id).await?;
        let mut tx = self.pool.begin().await?;
        let ids:Vec<String>=sqlx::query_scalar("SELECT id FROM meal_cooked_event WHERE meal_id=? ORDER BY local_date DESC,created_at DESC,id").bind(meal_id).fetch_all(&mut *tx).await?;
        let mut events = vec![];
        for id in ids {
            events.push(load_event(&mut tx, &id).await?);
        }
        let last_cooked = events
            .iter()
            .filter(|e| !e.voided)
            .map(|e| e.local_date.clone())
            .max();
        tx.commit().await?;
        Ok(MealHistory {
            meal_id: meal_id.into(),
            last_cooked,
            events,
        })
    }
    pub async fn change_history(&self, input: HistoryMutation) -> AppResult<MealHistory> {
        validation::id(&input.meal_id)?;
        validation::id(&input.action_id)?;
        let request = serde_json::to_string(&serde_json::json!({"scope":"history","input":input}))
            .map_err(|_| AppError::validation("Invalid history action."))?;
        let mut tx = self.pool.begin().await?;
        if replay(&mut tx, &input.action_id, &request).await? {
            tx.commit().await?;
            return self.meal_history(&input.meal_id).await;
        }
        let _: String = sqlx::query_scalar("SELECT id FROM meal WHERE id=?")
            .bind(&input.meal_id)
            .fetch_one(&mut *tx)
            .await?;
        match &input.operation {
            HistoryOperation::Record {
                event_id,
                local_date: date,
            } => {
                validation::id(event_id)?;
                validate_cooked_date(&mut tx, date).await?;
                sqlx::query("INSERT INTO meal_cooked_event(id,meal_id,local_date) VALUES (?,?,?)")
                    .bind(event_id)
                    .bind(&input.meal_id)
                    .bind(date)
                    .execute(&mut *tx)
                    .await?;
            }
            operation => {
                let (event_id, expected_revision, confirmed, date) = match operation {
                    HistoryOperation::Correct {
                        event_id,
                        expected_revision,
                        local_date,
                        confirmed,
                    } => (event_id, *expected_revision, *confirmed, Some(local_date)),
                    HistoryOperation::Undo {
                        event_id,
                        expected_revision,
                        confirmed,
                    } => (event_id, *expected_revision, *confirmed, None),
                    _ => unreachable!(),
                };
                validation::id(event_id)?;
                confirmation(confirmed)?;
                let event = load_event(&mut tx, event_id).await?;
                if event.meal_id != input.meal_id
                    || event.revision != expected_revision
                    || event.voided
                {
                    return Err(AppError::conflict());
                }
                let next = date.unwrap_or(&event.local_date);
                if date.is_some() {
                    validate_cooked_date(&mut tx, next).await?;
                } else {
                    local_date(next)?;
                }
                change_event(&mut tx, &event, next, date.is_none()).await?;
                if let Some(plan_id) = event.plan_entry_id {
                    let week: String =
                        sqlx::query_scalar("SELECT week_start FROM plan_entry WHERE id=?")
                            .bind(&plan_id)
                            .fetch_one(&mut *tx)
                            .await?;
                    sqlx::query("UPDATE plan_entry SET status=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?").bind(if date.is_none(){"planned"}else{"cooked"}).bind(plan_id).execute(&mut *tx).await?;
                    bump_week(&mut tx, &week).await?;
                }
            }
        }
        receipt(&mut tx, &input.action_id, &request).await?;
        tx.commit().await?;
        self.meal_history(&input.meal_id).await
    }
}
