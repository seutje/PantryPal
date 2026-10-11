use super::{stored_quantity, Context, ShoppingItem, Source};
use crate::{
    domain::{
        catalog::Unit,
        measurements::{
            self, rational::Rational, ConversionRequest, ConversionResult, Mode, Purpose,
        },
        planning::PlanStatus,
        validation,
    },
    error::{AppError, AppResult},
};
fn ratio(a: &validation::Amount) -> AppResult<Rational> {
    Rational::new(a.numerator.into(), a.denominator.into())
}
fn convert_one(
    from: &str,
    to: &str,
    ingredient: &str,
    ctx: &Context,
) -> AppResult<Option<(Rational, bool, Vec<String>)>> {
    match measurements::convert(
        ConversionRequest {
            amount: "1".into(),
            from_unit: from.into(),
            to_unit: to.into(),
            ingredient_id: Some(ingredient.into()),
            purpose: Purpose::Quantity,
        },
        &ctx.units,
        &[],
        &ctx.rules,
        &ctx.prefs,
    )? {
        ConversionResult::Converted {
            numerator,
            denominator,
            approximate,
            rule_ids,
            ..
        } => Ok(Some((
            Rational::new(
                numerator.parse().map_err(|_| AppError::storage())?,
                denominator.parse().map_err(|_| AppError::storage())?,
            )?,
            approximate,
            rule_ids,
        ))),
        ConversionResult::CannotConvert { .. } => Ok(None),
    }
}
pub(super) fn unit<'a>(ctx: &'a Context, id: &str) -> AppResult<&'a Unit> {
    ctx.units
        .iter()
        .find(|u| u.id == id)
        .ok_or_else(AppError::storage)
}
fn warn(item: &mut ShoppingItem, text: &str) {
    if !item.warnings.iter().any(|w| w == text) {
        item.warnings.push(text.into());
    }
}
pub fn generate(ctx: &Context) -> AppResult<Vec<ShoppingItem>> {
    let mut output: Vec<ShoppingItem> = vec![];
    // Stable anchor priority, independent of display preferences and plan order.
    let mut anchors: Vec<&Unit> = ctx
        .units
        .iter()
        .filter(|u| u.dimension != "temperature" && u.dimension != "informal")
        .collect();
    anchors.sort_by_key(|u| {
        (
            if u.id == "g" {
                0
            } else if u.id == "ml" {
                1
            } else {
                2
            },
            u.id.clone(),
        )
    });
    for entry in ctx
        .entries
        .iter()
        .filter(|e| e.status != PlanStatus::Skipped)
    {
        let meal = ctx
            .meals
            .iter()
            .find(|m| m.id == entry.meal_id)
            .ok_or_else(AppError::storage)?;
        let scale = ratio(&entry.servings)?.checked_div(ratio(&meal.data.base_servings)?)?;
        for line in &meal.data.ingredients {
            let ingredient = line
                .ingredient_id
                .as_ref()
                .and_then(|id| ctx.ingredients.iter().find(|i| &i.id == id));
            let mut item = ShoppingItem {
                id: String::new(),
                key: None,
                ingredient_id: line.ingredient_id.clone(),
                label: ingredient
                    .map(|i| i.data.name.clone())
                    .unwrap_or_else(|| line.raw_text.clone()),
                section_id: ingredient.and_then(|i| i.data.section_id.clone()),
                numerator: None,
                denominator: None,
                unit_id: line.unit_id.clone(),
                amount_text: line
                    .amount_text
                    .clone()
                    .unwrap_or_else(|| "Unknown amount".into()),
                approximate: false,
                warnings: vec![],
                rule_ids: vec![],
                sources: vec![Source {
                    entry_id: entry.id.clone(),
                    meal_id: meal.id.clone(),
                    meal_title: meal.data.title.clone(),
                    planned_servings: entry.servings.clone(),
                    base_servings: meal.data.base_servings.clone(),
                    line: line.clone(),
                }],
                adjustment: None,
                checked: false,
                excluded: false,
                orphaned: false,
                legacy: None,
            };
            if line.optional {
                warn(&mut item, "Includes an optional recipe ingredient.");
            }
            let source_unit = line
                .unit_id
                .as_deref()
                .map(|id| unit(ctx, id))
                .transpose()?;
            let mut group = format!("line:{}:{}", entry.id, line.id);
            if ingredient.is_none() {
                warn(&mut item,"Unresolved ingredient; confirm its canonical identity in Meals. Kept separate.");
            }
            if let Some(q) = &line.quantity {
                let mut value = ratio(q)?.checked_mul(scale)?;
                if source_unit.is_some_and(|u| u.dimension == "temperature") {
                    value = ratio(q)?; // Temperature cannot scale with portions.
                    warn(
                        &mut item,
                        "Temperature is not a grocery quantity. Kept separate.",
                    );
                } else if source_unit.is_some_and(|u| u.dimension == "informal") {
                    warn(
                        &mut item,
                        "Informal measure; kept separate and scaled only as written.",
                    );
                } else if let (Some(ingredient), Some(from)) = (ingredient, source_unit) {
                    let anchor = if ctx.prefs.aggregate_compatible {
                        let mut found = None;
                        for candidate in &anchors {
                            if let Some(conversion) =
                                convert_one(&from.id, &candidate.id, &ingredient.id, ctx)?
                            {
                                found = Some((*candidate, conversion));
                                break;
                            }
                        }
                        found
                    } else {
                        Some((from, (Rational::new(1, 1)?, false, vec![])))
                    };
                    if let Some((anchor, (factor, approx, rules))) = anchor {
                        value = value.checked_mul(factor)?;
                        item.unit_id = Some(anchor.id.clone());
                        item.approximate = approx;
                        item.rule_ids = rules;
                        group = format!("ingredient:{}:group:{}", ingredient.id, anchor.id);
                    }
                } else if source_unit.is_none() {
                    warn(&mut item, "Missing unit; kept separate.");
                }
                item.numerator = Some(value.n.to_string());
                item.denominator = Some(value.d.to_string());
            } else {
                warn(
                    &mut item,
                    "Unknown or to-taste quantity; not numerically scaled or merged.",
                );
            }
            item.key = Some(group.clone());
            item.id = group;
            if let Some(old) = output.iter_mut().find(|old| old.key == item.key) {
                let a = stored_quantity(old)?.ok_or_else(AppError::storage)?;
                let b = stored_quantity(&item)?.ok_or_else(AppError::storage)?;
                let total = a.checked_add(b)?;
                old.numerator = Some(total.n.to_string());
                old.denominator = Some(total.d.to_string());
                old.approximate |= item.approximate;
                for warning in &item.warnings {
                    warn(old, warning);
                }
                for rule in item.rule_ids {
                    if !old.rule_ids.contains(&rule) {
                        old.rule_ids.push(rule);
                    }
                }
                old.sources.extend(item.sources);
            } else {
                output.push(item);
            }
        }
    }
    for item in &mut output {
        let Some(ingredient) = item
            .ingredient_id
            .as_ref()
            .and_then(|id| ctx.ingredients.iter().find(|i| &i.id == id))
        else {
            continue;
        };
        let Some(from) = item.unit_id.clone() else {
            continue;
        };
        if item
            .key
            .as_ref()
            .is_none_or(|k| !k.starts_with("ingredient:"))
        {
            continue;
        }
        let dimension = &unit(ctx, &from)?.dimension;
        let global = match (&ctx.prefs.mode, dimension.as_str()) {
            (Mode::Metric, "mass") => Some("g"),
            (Mode::Metric, "volume") => Some("ml"),
            (Mode::Us, "mass") => Some("oz"),
            (Mode::Us, "volume") => Some("cup-us"),
            _ => None,
        };
        let desired = ingredient
            .data
            .preferred_unit_id
            .as_deref()
            .or(global)
            .unwrap_or(&from);
        // Convert every ORIGINAL source directly to the display target. Going
        // via the grouping anchor could accidentally chain two ingredient rules.
        let total_in = |target: &str| -> AppResult<Option<(Rational, bool, Vec<String>)>> {
            let mut total = Rational::new(0, 1)?;
            let mut approximate = false;
            let mut rules = vec![];
            for source in &item.sources {
                let Some(source_unit) = source.line.unit_id.as_deref() else {
                    return Ok(None);
                };
                let Some((factor, approx, used)) =
                    convert_one(source_unit, target, &ingredient.id, ctx)?
                else {
                    return Ok(None);
                };
                let q = ratio(
                    source
                        .line
                        .quantity
                        .as_ref()
                        .ok_or_else(AppError::storage)?,
                )?
                .checked_mul(ratio(&source.planned_servings)?)?
                .checked_div(ratio(&source.base_servings)?)?
                .checked_mul(factor)?;
                total = total.checked_add(q)?;
                approximate |= approx;
                for rule in used {
                    if !rules.contains(&rule) {
                        rules.push(rule);
                    }
                }
            }
            Ok(Some((total, approximate, rules)))
        };
        let requested = total_in(desired)?;
        let unavailable = requested.is_none();
        let target = if let Some(result) = requested {
            Some((desired.to_owned(), result))
        } else if let Some(global) = global {
            total_in(global)?.map(|result| (global.to_owned(), result))
        } else {
            None
        };
        if let Some((target, (value, approximate, rules))) = target {
            item.numerator = Some(value.n.to_string());
            item.denominator = Some(value.d.to_string());
            item.unit_id = Some(target);
            item.approximate = approximate;
            item.rule_ids = rules;
        }
        if unavailable {
            warn(item,"Preferred unit has no unique direct conversion path for every source; using a compatible unit.");
        }
    }
    let identities: Vec<_> = output
        .iter()
        .filter_map(|i| i.ingredient_id.clone())
        .collect();
    for item in &mut output {
        if item
            .ingredient_id
            .as_ref()
            .is_some_and(|id| identities.iter().filter(|x| *x == id).count() > 1)
        {
            warn(
                item,
                "Separate quantity: unknown, informal, incompatible or ambiguous conversion path.",
            );
        }
    }
    Ok(output)
}
