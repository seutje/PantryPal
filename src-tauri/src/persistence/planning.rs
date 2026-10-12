use super::{
    catalog::confirmation,
    history::{change_event, load_event, validate_cooked_date},
    Database,
};
use crate::{
    domain::{planning::*, validation},
    error::{AppError, AppResult},
};
use sqlx::{Row, SqliteConnection};
use uuid::Uuid;

pub(crate) async fn replay(
    connection: &mut SqliteConnection,
    id: &str,
    request: &str,
) -> AppResult<bool> {
    validation::id(id)?;
    let old: Option<String> =
        sqlx::query_scalar("SELECT request_json FROM planner_action WHERE id=?")
            .bind(id)
            .fetch_optional(&mut *connection)
            .await?;
    match old {
        Some(old) if old == request => Ok(true),
        Some(_) => Err(AppError::conflict()),
        None => Ok(false),
    }
}
pub(crate) async fn receipt(
    connection: &mut SqliteConnection,
    id: &str,
    request: &str,
) -> AppResult<()> {
    sqlx::query("INSERT INTO planner_action(id,request_json) VALUES (?,?)")
        .bind(id)
        .bind(request)
        .execute(connection)
        .await?;
    Ok(())
}
pub(crate) async fn bump_week(connection: &mut SqliteConnection, week: &str) -> AppResult<()> {
    sqlx::query("INSERT INTO plan_week(week_start,revision) VALUES (?,1) ON CONFLICT(week_start) DO UPDATE SET revision=revision+1").bind(week).execute(connection).await?;
    Ok(())
}
async fn append_order(
    connection: &mut SqliteConnection,
    week: &str,
    date: &Option<String>,
) -> AppResult<i64> {
    Ok(sqlx::query_scalar("SELECT coalesce(max(sort_order)+1,0) FROM plan_entry WHERE week_start=? AND local_date IS ?").bind(week).bind(date).fetch_one(connection).await?)
}
pub(crate) async fn load_entry(
    connection: &mut SqliteConnection,
    id: &str,
) -> AppResult<PlanEntry> {
    validation::id(id)?;
    let row = sqlx::query("SELECT * FROM plan_entry WHERE id=?")
        .bind(id)
        .fetch_one(&mut *connection)
        .await?;
    let amount = crate::domain::validation::Amount {
        numerator: row.try_get("servings_numerator")?,
        denominator: row.try_get("servings_denominator")?,
    };
    let event_id: Option<String> =
        sqlx::query_scalar("SELECT id FROM meal_cooked_event WHERE plan_entry_id=?")
            .bind(id)
            .fetch_optional(&mut *connection)
            .await?;
    let cooked_event = if let Some(id) = event_id {
        Some(load_event(connection, &id).await?)
    } else {
        None
    };
    let status = match row.try_get::<&str, _>("status")? {
        "planned" => PlanStatus::Planned,
        "cooked" => PlanStatus::Cooked,
        "skipped" => PlanStatus::Skipped,
        _ => return Err(AppError::storage()),
    };
    Ok(PlanEntry {
        id: row.try_get("id")?,
        meal_id: row.try_get("meal_id")?,
        local_date: row.try_get("local_date")?,
        servings_text: row
            .try_get::<Option<String>, _>("servings_text")?
            .unwrap_or_else(|| amount_text(&amount)),
        servings: amount,
        quantity_displays: vec![],
        status,
        sort_order: row.try_get("sort_order")?,
        cooked_event,
    })
}
impl Database {
    pub(crate) async fn week_entries(&self, week: &str) -> AppResult<(i64, Vec<PlanEntry>)> {
        validate_week(week)?;
        let mut tx = self.pool.begin().await?;
        let revision: i64 = sqlx::query_scalar(
            "SELECT coalesce((SELECT revision FROM plan_week WHERE week_start=?),0)",
        )
        .bind(week)
        .fetch_one(&mut *tx)
        .await?;
        let ids: Vec<String> = sqlx::query_scalar(
            "SELECT id FROM plan_entry WHERE week_start=? ORDER BY local_date,sort_order,id",
        )
        .bind(week)
        .fetch_all(&mut *tx)
        .await?;
        let mut entries = vec![];
        for id in ids {
            entries.push(load_entry(&mut tx, &id).await?);
        }
        tx.commit().await?;
        Ok((revision, entries))
    }
    pub(crate) async fn mutate_plan(&self, input: &PlanMutation) -> AppResult<()> {
        validate_week(&input.week_start)?;
        validation::id(&input.action_id)?;
        let request = serde_json::to_string(&serde_json::json!({"scope":"plan","input":input}))
            .map_err(|_| AppError::validation("Invalid plan action."))?;
        let mut tx = self.pool.begin().await?;
        if replay(&mut tx, &input.action_id, &request).await? {
            tx.commit().await?;
            return Ok(());
        }
        let revision: i64 = sqlx::query_scalar(
            "SELECT coalesce((SELECT revision FROM plan_week WHERE week_start=?),0)",
        )
        .bind(&input.week_start)
        .fetch_one(&mut *tx)
        .await?;
        if revision != input.expected_revision {
            return Err(AppError::conflict());
        }
        match &input.operation {
            PlanOperation::Add {
                entry_id,
                meal_id,
                local_date,
                servings: text,
            } => {
                validation::id(entry_id)?;
                validation::id(meal_id)?;
                validate_day(&input.week_start, local_date)?;
                let amount = servings(text)?;
                let retired: bool = sqlx::query_scalar("SELECT retired FROM meal WHERE id=?")
                    .bind(meal_id)
                    .fetch_one(&mut *tx)
                    .await?;
                if retired {
                    return Err(AppError::validation(
                        "Retired meals cannot be added to a new plan.",
                    ));
                }
                let count: i64 =
                    sqlx::query_scalar("SELECT count(*) FROM plan_entry WHERE week_start=?")
                        .bind(&input.week_start)
                        .fetch_one(&mut *tx)
                        .await?;
                if count >= 500 {
                    return Err(AppError::validation("This week already has 500 entries."));
                }
                let order = append_order(&mut tx, &input.week_start, local_date).await?;
                sqlx::query("INSERT INTO plan_entry(id,week_start,local_date,meal_id,servings_numerator,servings_denominator,sort_order,servings_text) VALUES (?,?,?,?,?,?,?,?)").bind(entry_id).bind(&input.week_start).bind(local_date).bind(meal_id).bind(amount.numerator).bind(amount.denominator).bind(order).bind(text).execute(&mut *tx).await?;
            }
            PlanOperation::Reorder {
                local_date,
                entry_ids,
            } => {
                validate_day(&input.week_start, local_date)?;
                if entry_ids.len() > 500 {
                    return Err(AppError::validation("Too many entries."));
                }
                let current: Vec<String> = sqlx::query_scalar(
                    "SELECT id FROM plan_entry WHERE week_start=? AND local_date IS ?",
                )
                .bind(&input.week_start)
                .bind(local_date)
                .fetch_all(&mut *tx)
                .await?;
                let unique: std::collections::HashSet<_> = entry_ids.iter().collect();
                for id in entry_ids {
                    validation::id(id)?;
                }
                if unique.len() != entry_ids.len()
                    || current.len() != entry_ids.len()
                    || !current.iter().all(|id| unique.contains(id))
                {
                    return Err(AppError::conflict());
                }
                for (order, id) in entry_ids.iter().enumerate() {
                    sqlx::query("UPDATE plan_entry SET sort_order=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?").bind(order as i64).bind(id).execute(&mut *tx).await?;
                }
            }
            operation => {
                let id = match operation {
                    PlanOperation::Edit { entry_id, .. }
                    | PlanOperation::Remove { entry_id, .. }
                    | PlanOperation::SetStatus { entry_id, .. } => entry_id,
                    _ => unreachable!(),
                };
                validation::id(id)?;
                let week: String =
                    sqlx::query_scalar("SELECT week_start FROM plan_entry WHERE id=?")
                        .bind(id)
                        .fetch_one(&mut *tx)
                        .await?;
                if week != input.week_start {
                    return Err(AppError::conflict());
                }
                let entry = load_entry(&mut tx, id).await?;
                match operation {
                    PlanOperation::Edit {
                        local_date,
                        servings: text,
                        ..
                    } => {
                        validate_day(&week, local_date)?;
                        let amount = if *text == entry.servings_text {
                            entry.servings.clone()
                        } else {
                            servings(text)?
                        };
                        let order = if *local_date == entry.local_date {
                            entry.sort_order
                        } else {
                            append_order(&mut tx, &week, local_date).await?
                        };
                        sqlx::query("UPDATE plan_entry SET local_date=?,servings_numerator=?,servings_denominator=?,servings_text=?,sort_order=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?").bind(local_date).bind(amount.numerator).bind(amount.denominator).bind(text).bind(order).bind(id).execute(&mut *tx).await?;
                    }
                    PlanOperation::Remove { confirmed, .. } => {
                        confirmation(*confirmed)?;
                        // Detach history instead of erasing an actual cooked meal.
                        sqlx::query("UPDATE meal_cooked_event SET plan_entry_id=NULL,revision=revision+1 WHERE plan_entry_id=?").bind(id).execute(&mut *tx).await?;
                        sqlx::query("DELETE FROM plan_entry WHERE id=?")
                            .bind(id)
                            .execute(&mut *tx)
                            .await?;
                    }
                    PlanOperation::SetStatus {
                        status,
                        cooked_date,
                        confirmed,
                        ..
                    } => {
                        if *status == PlanStatus::Cooked {
                            let date = cooked_date.as_ref().ok_or_else(|| {
                                AppError::validation("Choose the date actually cooked.")
                            })?;
                            validate_cooked_date(&mut tx, date).await?;
                            if let Some(event) = entry.cooked_event {
                                if !event.voided && event.local_date != *date {
                                    confirmation(*confirmed)?;
                                }
                                if event.voided || event.local_date != *date {
                                    change_event(&mut tx, &event, date, false).await?;
                                }
                            } else {
                                sqlx::query("INSERT INTO meal_cooked_event(id,meal_id,local_date,plan_entry_id,origin_plan_entry_id) VALUES (?,?,?,?,?)").bind(Uuid::new_v4().to_string()).bind(&entry.meal_id).bind(date).bind(id).bind(id).execute(&mut *tx).await?;
                            }
                        } else {
                            if cooked_date.is_some() {
                                return Err(AppError::validation(
                                    "Only cooked entries have a cooked date.",
                                ));
                            }
                            if let Some(event) = entry.cooked_event.filter(|e| !e.voided) {
                                confirmation(*confirmed)?;
                                change_event(&mut tx, &event, &event.local_date, true).await?;
                            }
                        }
                        sqlx::query("UPDATE plan_entry SET status=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?").bind(status.as_str()).bind(id).execute(&mut *tx).await?;
                    }
                    _ => unreachable!(),
                }
            }
        }
        bump_week(&mut tx, &input.week_start).await?;
        receipt(&mut tx, &input.action_id, &request).await?;
        tx.commit().await?;
        Ok(())
    }
}
