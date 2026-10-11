use super::{meal_library::MealLibrary, measurements::rational, validation::Amount};
use crate::error::{AppError, AppResult};
use chrono::{Datelike, Days, NaiveDate};
use serde::{Deserialize, Serialize};

pub fn local_date(text: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .ok()
        .filter(|date| {
            date.format("%Y-%m-%d").to_string() == text && (1..=9999).contains(&date.year())
        })
        .ok_or_else(|| AppError::validation("Choose a valid local date (YYYY-MM-DD)."))
}
pub fn week_dates(date: &str) -> AppResult<Vec<String>> {
    let date = local_date(date)?;
    let monday = date
        .checked_sub_days(Days::new(date.weekday().num_days_from_monday().into()))
        .ok_or_else(|| AppError::validation("This week is outside the supported date range."))?;
    (0..7)
        .map(|offset| {
            monday
                .checked_add_days(Days::new(offset))
                .filter(|d| (1..=9999).contains(&d.year()))
                .map(|d| d.format("%Y-%m-%d").to_string())
                .ok_or_else(|| {
                    AppError::validation("This week is outside the supported date range.")
                })
        })
        .collect()
}
pub fn validate_week(week: &str) -> AppResult<Vec<String>> {
    let dates = week_dates(week)?;
    if dates[0] != week {
        return Err(AppError::validation("Weeks must start on Monday."));
    }
    Ok(dates)
}
pub fn validate_day(week: &str, day: &Option<String>) -> AppResult<()> {
    let dates = validate_week(week)?;
    if let Some(day) = day {
        local_date(day)?;
        if day < &dates[0] || day > &dates[6] {
            return Err(AppError::validation(
                "Choose a date in this week, or This week (unscheduled).",
            ));
        }
    }
    Ok(())
}
pub fn servings(text: &str) -> AppResult<Amount> {
    let q = rational::parse(text)?.ok_or_else(|| {
        AppError::validation("Planned servings must be known and greater than zero.")
    })?;
    let amount = Amount {
        numerator: q.n as i64,
        denominator: q.d as i64,
    };
    amount.validate(true)?;
    Ok(amount)
}
pub fn amount_text(a: &Amount) -> String {
    if a.denominator == 1 {
        a.numerator.to_string()
    } else {
        format!("{}/{}", a.numerator, a.denominator)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Planned,
    Cooked,
    Skipped,
}
impl PlanStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Planned => "planned",
            Self::Cooked => "cooked",
            Self::Skipped => "skipped",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanMutation {
    pub action_id: String,
    pub week_start: String,
    pub expected_revision: i64,
    pub operation: PlanOperation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum PlanOperation {
    Add {
        entry_id: String,
        meal_id: String,
        local_date: Option<String>,
        servings: String,
    },
    Edit {
        entry_id: String,
        local_date: Option<String>,
        servings: String,
    },
    Reorder {
        local_date: Option<String>,
        entry_ids: Vec<String>,
    },
    Remove {
        entry_id: String,
        confirmed: bool,
    },
    SetStatus {
        entry_id: String,
        status: PlanStatus,
        cooked_date: Option<String>,
        confirmed: bool,
    },
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CookedEvent {
    pub id: String,
    pub meal_id: String,
    pub local_date: String,
    pub plan_entry_id: Option<String>,
    pub origin_plan_entry_id: Option<String>,
    pub voided: bool,
    pub revision: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanEntry {
    pub id: String,
    pub meal_id: String,
    pub local_date: Option<String>,
    pub servings: Amount,
    pub servings_text: String,
    pub status: PlanStatus,
    pub sort_order: i64,
    pub cooked_event: Option<CookedEvent>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekView {
    pub week_start: String,
    pub week_end: String,
    pub days: Vec<String>,
    pub previous_week: Option<String>,
    pub next_week: Option<String>,
    pub revision: i64,
    pub entries: Vec<PlanEntry>,
    pub library: MealLibrary,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MealHistory {
    pub meal_id: String,
    pub last_cooked: Option<String>,
    pub events: Vec<CookedEvent>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoryMutation {
    pub action_id: String,
    pub meal_id: String,
    pub operation: HistoryOperation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum HistoryOperation {
    Record {
        event_id: String,
        local_date: String,
    },
    Correct {
        event_id: String,
        expected_revision: i64,
        local_date: String,
        confirmed: bool,
    },
    Undo {
        event_id: String,
        expected_revision: i64,
        confirmed: bool,
    },
}
