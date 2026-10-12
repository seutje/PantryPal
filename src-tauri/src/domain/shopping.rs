mod display;
mod generation;
use super::{
    catalog::{Ingredient, Unit, Vocabulary},
    meals::Meal,
    meals::MealIngredient,
    measurements::{
        rational::{parse, Rational},
        ConversionRule, MeasurementPreferences,
    },
    planning::PlanEntry,
    validation,
};
use crate::error::{AppError, AppResult};
pub use generation::generate;
use generation::unit;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub entry_id: String,
    pub meal_id: String,
    pub meal_title: String,
    pub planned_servings: validation::Amount,
    pub base_servings: validation::Amount,
    pub line: MealIngredient,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Override {
    pub amount: String,
    pub unit_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShoppingItem {
    pub id: String,
    pub key: Option<String>,
    pub ingredient_id: Option<String>,
    pub label: String,
    pub section_id: Option<String>,
    pub numerator: Option<String>,
    pub denominator: Option<String>,
    pub unit_id: Option<String>,
    pub amount_text: String,
    pub approximate: bool,
    pub warnings: Vec<String>,
    pub rule_ids: Vec<String>,
    pub sources: Vec<Source>,
    pub adjustment: Option<Override>,
    pub checked: bool,
    pub excluded: bool,
    pub orphaned: bool,
    pub legacy: Option<serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub version: u8,
    pub items: Vec<ShoppingItem>,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            items: vec![],
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShoppingView {
    pub week_start: String,
    pub revision: i64,
    pub generated: bool,
    pub stale: bool,
    pub items: Vec<DisplayItem>,
    pub sections: Vec<Vocabulary>,
    pub units: Vec<Unit>,
    pub text: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayItem {
    #[serde(flatten)]
    pub item: ShoppingItem,
    pub display: String,
    pub rounded: bool,
    pub alternatives: Vec<String>,
    pub section: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShoppingMutation {
    pub action_id: String,
    pub week_start: String,
    pub expected_revision: i64,
    pub operation: ShoppingOperation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ShoppingOperation {
    Refresh,
    Manual {
        item_id: String,
        label: String,
        amount: String,
        unit_id: Option<String>,
        section_id: Option<String>,
    },
    Adjust {
        item_id: String,
        amount: String,
        unit_id: Option<String>,
    },
    Reset {
        item_id: String,
        confirmed: bool,
    },
    Check {
        item_id: String,
        checked: bool,
    },
    Exclude {
        item_id: String,
        excluded: bool,
    },
}
pub struct Context {
    pub entries: Vec<PlanEntry>,
    pub meals: Vec<Meal>,
    pub ingredients: Vec<Ingredient>,
    pub units: Vec<Unit>,
    pub sections: Vec<Vocabulary>,
    pub rules: Vec<ConversionRule>,
    pub prefs: MeasurementPreferences,
}
pub fn merge(old: &Document, generated: Vec<ShoppingItem>) -> Document {
    let mut items = generated;
    for previous in &old.items {
        if previous.key.is_none() {
            items.push(previous.clone());
            continue;
        }
        if let Some(current) = items.iter_mut().find(|i| i.key == previous.key) {
            current.id = previous.id.clone();
            current.adjustment = previous.adjustment.clone();
            current.checked = previous.checked;
            current.excluded = previous.excluded;
        } else if previous.adjustment.is_some()
            || previous.checked
            || previous.excluded
            || previous.legacy.is_some()
        {
            let mut retained = previous.clone();
            retained.orphaned = true;
            items.push(retained);
        }
    }
    Document { version: 1, items }
}
pub fn validate_override(value: &Override, ctx: &Context) -> AppResult<()> {
    validation::text(&value.amount, 120, false)?;
    if parse(&value.amount)?.is_some_and(|q| q.n < 0) {
        return Err(AppError::validation("Shopping amounts cannot be negative."));
    }
    validation::optional_id(&value.unit_id)?;
    if let Some(id) = &value.unit_id {
        let selected =
            ctx.units.iter().find(|u| &u.id == id).ok_or_else(|| {
                AppError::validation("Choose a registered grocery quantity unit.")
            })?;
        if selected.dimension == "temperature" {
            return Err(AppError::validation("Choose a grocery quantity unit."));
        }
    }
    Ok(())
}
fn stored_quantity(item: &ShoppingItem) -> AppResult<Option<Rational>> {
    match (&item.numerator, &item.denominator) {
        (Some(n), Some(d)) => Ok(Some(Rational::new(
            n.parse().map_err(|_| AppError::storage())?,
            d.parse().map_err(|_| AppError::storage())?,
        )?)),
        (None, None) => Ok(None),
        _ => Err(AppError::storage()),
    }
}
pub fn view(
    week: String,
    revision: i64,
    generated: bool,
    stale: bool,
    doc: Document,
    ctx: &Context,
) -> AppResult<ShoppingView> {
    let mut items = vec![];
    for item in doc.items {
        let (quantity, raw, unit_id) = if let Some(adjustment) = &item.adjustment {
            (
                parse(&adjustment.amount)?,
                adjustment.amount.clone(),
                adjustment.unit_id.as_ref(),
            )
        } else {
            (
                stored_quantity(&item)?,
                item.amount_text.clone(),
                item.unit_id.as_ref(),
            )
        };
        let selected_unit = unit_id.map(|id| unit(ctx, id)).transpose()?;
        let (display, rounded, alternatives) = if let Some(q) = quantity {
            display::format(q, selected_unit, ctx)?
        } else {
            let amount = if raw.trim().is_empty() {
                "Unknown amount".into()
            } else {
                raw
            };
            let symbol = selected_unit.map(|u| u.symbol.as_str()).unwrap_or_default();
            let display = if amount.eq_ignore_ascii_case(symbol) {
                amount
            } else {
                format!("{amount} {symbol}").trim().into()
            };
            (display, false, vec![])
        };
        let section = item
            .section_id
            .as_ref()
            .and_then(|id| ctx.sections.iter().find(|s| &s.id == id))
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Other".into());
        items.push(DisplayItem {
            item,
            display,
            rounded,
            alternatives,
            section,
        });
    }
    items.sort_by_key(|i| {
        (
            i.item
                .section_id
                .as_ref()
                .and_then(|id| ctx.sections.iter().find(|s| &s.id == id))
                .map(|s| s.sort_order)
                .unwrap_or(i64::MAX),
            i.section.to_lowercase(),
            i.item.label.to_lowercase(),
            i.item.id.clone(),
        )
    });
    let clean = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut text = format!("Shopping · week of {week}");
    let mut section = String::new();
    for row in items
        .iter()
        .filter(|i| !i.item.excluded && !i.item.orphaned)
    {
        if section != row.section {
            section = row.section.clone();
            text.push_str(&format!("\r\n\r\n{}", clean(&section)));
        }
        text.push_str(&format!(
            "\r\n[{}] {} — {}{}{}{}",
            if row.item.checked { "x" } else { " " },
            clean(&row.item.label),
            clean(&row.display),
            if row.item.approximate {
                " (approximate)"
            } else {
                ""
            },
            if row.rounded { " (rounded up)" } else { "" },
            if row.item.sources.iter().any(|s| s.line.optional) {
                " (optional)"
            } else {
                ""
            }
        ));
        if !row.alternatives.is_empty() {
            text.push_str(&format!(" ({})", row.alternatives.join("; ")));
        }
    }
    Ok(ShoppingView {
        week_start: week,
        revision,
        generated,
        stale,
        items,
        sections: ctx.sections.clone(),
        units: ctx.units.clone(),
        text,
    })
}
