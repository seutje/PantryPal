pub mod rational;
use super::{catalog::Unit, validation};
use crate::error::{AppError, AppResult};
use rational::{parse, Rational};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeasurementPreferences {
    pub mode: Mode,
    pub cup_convention: Option<CupConvention>,
    pub fractions: bool,
    pub decimal_places: u8,
    pub aggregate_compatible: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Mixed,
    Metric,
    Us,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CupConvention {
    Metric,
    Us,
    Imperial,
}
impl MeasurementPreferences {
    pub fn validate(&self) -> AppResult<()> {
        if self.decimal_places > 6 {
            return Err(AppError::validation("Choose zero to six decimal places."));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversionRequest {
    pub amount: String,
    pub from_unit: String,
    pub to_unit: String,
    pub ingredient_id: Option<String>,
    pub purpose: Purpose,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    Quantity,
    Temperature,
}
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ConversionRule {
    pub id: String,
    pub ingredient_id: String,
    pub from_unit_id: String,
    pub from_numerator: i64,
    pub from_denominator: i64,
    pub to_unit_id: String,
    pub to_numerator: i64,
    pub to_denominator: i64,
    pub approximate: bool,
    pub notes: String,
    pub original_from_text: String,
    pub original_to_text: String,
    pub retired: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuleInput {
    pub ingredient_id: String,
    pub from_unit_id: String,
    pub from_amount: String,
    pub to_unit_id: String,
    pub to_amount: String,
    pub approximate: bool,
    pub notes: String,
}
#[derive(Debug, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ConversionResult {
    Converted {
        original: ConversionRequest,
        numerator: String,
        denominator: String,
        exact: String,
        display: String,
        rounded: bool,
        approximate: bool,
        rule_ids: Vec<String>,
        from_unit_id: String,
        to_unit_id: String,
    },
    CannotConvert {
        original: ConversionRequest,
        reason: String,
        candidates: Vec<String>,
    },
}
pub fn compatible(from: &Unit, to: &Unit) -> bool {
    from.id == to.id
        || (from.dimension == to.dimension
            && matches!(from.dimension.as_str(), "mass" | "volume" | "temperature"))
}
fn factor(unit: &Unit) -> AppResult<Rational> {
    Rational::new(
        i128::from(unit.factor_numerator.ok_or_else(AppError::storage)?),
        i128::from(unit.factor_denominator.ok_or_else(AppError::storage)?),
    )
}
fn offset(unit: &Unit) -> AppResult<Rational> {
    Rational::new(
        i128::from(unit.offset_numerator.ok_or_else(AppError::storage)?),
        i128::from(unit.offset_denominator.ok_or_else(AppError::storage)?),
    )
}
fn universal(amount: Rational, from: &Unit, to: &Unit) -> AppResult<Rational> {
    if from.id == to.id {
        return Ok(amount);
    }
    let base = amount.checked_mul(factor(from)?)?;
    if from.dimension == "temperature" {
        base.checked_add(offset(from)?)?
            .checked_add(offset(to)?.negated())?
            .checked_div(factor(to)?)
    } else {
        base.checked_div(factor(to)?)
    }
}

/// Resolver never lets display preferences silently reinterpret the source.
/// Ambiguous aliases return candidates; the caller must explicitly select an ID.
pub fn resolve<'a>(value: &str, units: &'a [Unit], aliases: &[(String, String)]) -> Vec<&'a Unit> {
    if let Some(unit) = units.iter().find(|u| u.id == value) {
        return vec![unit];
    }
    let normalized = validation::normalized(value);
    units
        .iter()
        .filter(|u| {
            validation::normalized(&u.label) == normalized
                || validation::normalized(&u.symbol) == normalized
                || aliases
                    .iter()
                    .any(|(alias, id)| alias == &normalized && id == &u.id)
        })
        .collect()
}
pub fn convert(
    request: ConversionRequest,
    units: &[Unit],
    aliases: &[(String, String)],
    rules: &[ConversionRule],
    prefs: &MeasurementPreferences,
) -> AppResult<ConversionResult> {
    prefs.validate()?;
    validation::text(&request.from_unit, 120, true)?;
    validation::text(&request.to_unit, 120, true)?;
    validation::optional_id(&request.ingredient_id)?;
    let cannot = |reason: &str, candidates: Vec<String>| ConversionResult::CannotConvert {
        original: request.clone(),
        reason: reason.into(),
        candidates,
    };
    let Some(amount) = parse(&request.amount)? else {
        return Ok(cannot(
            "Quantity is unknown; keep the original wording.",
            vec![],
        ));
    };
    let from = resolve(&request.from_unit, units, aliases);
    let to = resolve(&request.to_unit, units, aliases);
    if from.len() != 1 || to.len() != 1 {
        return Ok(cannot(
            "Unit is unknown or ambiguous. Choose an explicit regional unit.",
            from.iter()
                .filter(|_| from.len() != 1)
                .chain(to.iter().filter(|_| to.len() != 1))
                .map(|u| u.id.clone())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
        ));
    }
    let (from, to) = (from[0], to[0]);
    let temperature = from.dimension == "temperature" || to.dimension == "temperature";
    if temperature
        && (request.purpose != Purpose::Temperature
            || from.dimension != "temperature"
            || to.dimension != "temperature")
        || !temperature && request.purpose == Purpose::Temperature
    {
        return Ok(cannot(
            "Temperature is a separate display conversion, not a grocery quantity.",
            vec![],
        ));
    }
    if amount.n < 0 && !temperature {
        return Err(AppError::validation(
            "Grocery quantities cannot be negative.",
        ));
    }
    if from.dimension == "informal" || to.dimension == "informal" {
        return Ok(cannot("Informal units have no numeric conversion.", vec![]));
    }
    let mut used = vec![];
    let mut approximate = false;
    let converted = if compatible(from, to) {
        universal(amount, from, to)?
    } else {
        let mut paths = vec![];
        for rule in rules
            .iter()
            .filter(|r| !r.retired && Some(&r.ingredient_id) == request.ingredient_id.as_ref())
        {
            let a = units
                .iter()
                .find(|u| u.id == rule.from_unit_id)
                .ok_or_else(AppError::storage)?;
            let b = units
                .iter()
                .find(|u| u.id == rule.to_unit_id)
                .ok_or_else(AppError::storage)?;
            let ra = Rational::new(
                i128::from(rule.from_numerator),
                i128::from(rule.from_denominator),
            )?;
            let rb = Rational::new(
                i128::from(rule.to_numerator),
                i128::from(rule.to_denominator),
            )?;
            for (a, b, ra, rb) in [(a, b, ra, rb), (b, a, rb, ra)] {
                if compatible(from, a) && compatible(b, to) {
                    paths.push((
                        universal(amount, from, a)?
                            .checked_div(ra)?
                            .checked_mul(rb)
                            .and_then(|x| universal(x, b, to))?,
                        rule,
                    ));
                }
            }
        }
        if paths.len() != 1 {
            return Ok(cannot(
                if paths.is_empty() {
                    "No applicable ingredient-specific conversion. Keep these quantities separate."
                } else {
                    "Multiple ingredient rules apply. Review the rules before converting."
                },
                paths.iter().map(|(_, r)| r.id.clone()).collect(),
            ));
        }
        let (value, rule) = paths[0];
        used.push(rule.id.clone());
        approximate = rule.approximate;
        value
    };
    let (display, rounded) = if prefs.fractions {
        (converted.display_fraction(), false)
    } else {
        converted.decimal(prefs.decimal_places)?
    };
    Ok(ConversionResult::Converted {
        original: request,
        numerator: converted.n.to_string(),
        denominator: converted.d.to_string(),
        exact: converted.fraction(),
        display,
        rounded,
        approximate,
        rule_ids: used,
        from_unit_id: from.id.clone(),
        to_unit_id: to.id.clone(),
    })
}

#[cfg(test)]
mod tests;
