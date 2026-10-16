use super::*;
use crate::domain::{meal_library::IngredientDraft, measurements::rational, validation};
use serde_json::Value;
use uuid::Uuid;
pub struct JsonLdImporter;
fn error() -> AppError {
    AppError::validation("Recipe JSON-LD is missing, malformed or ambiguous. Paste a single Recipe object or use manual entry.")
}
fn recipe_nodes<'a>(v: &'a Value, depth: usize, found: &mut Vec<&'a Value>) -> AppResult<()> {
    if depth > 24 {
        return Err(error());
    }
    match v {
        Value::Array(items) => {
            for item in items {
                recipe_nodes(item, depth + 1, found)?;
            }
        }
        Value::Object(obj) => {
            let is_recipe = match obj.get("@type") {
                Some(Value::String(s)) => s == "Recipe" || s == "https://schema.org/Recipe",
                Some(Value::Array(a)) => a.iter().any(|v| v == "Recipe"),
                _ => false,
            };
            if is_recipe {
                found.push(v);
            }
            if let Some(graph) = obj.get("@graph") {
                recipe_nodes(graph, depth + 1, found)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn string(v: &Value) -> String {
    v.as_str().map(str::to_owned).unwrap_or_default()
}
fn instructions(v: &Value, depth: usize) -> AppResult<Vec<String>> {
    if depth > 24 {
        return Err(error());
    }
    match v {
        Value::String(s) => Ok(vec![s.clone()]),
        Value::Array(a) => {
            let mut out = vec![];
            for item in a {
                out.extend(instructions(item, depth + 1)?);
            }
            Ok(out)
        }
        Value::Object(o) => {
            if let Some(section) = o.get("itemListElement") {
                let mut out = vec![];
                if let Some(name) = o.get("name").and_then(Value::as_str) {
                    out.push(name.into());
                }
                out.extend(instructions(section, depth + 1)?);
                Ok(out)
            } else if let Some(text) = o.get("text").and_then(Value::as_str) {
                Ok(vec![text.into()])
            } else {
                Err(error())
            }
        }
        Value::Null => Ok(vec![]),
        _ => Err(error()),
    }
}
fn minutes(v: &Value) -> Option<i64> {
    let text = v.as_str()?.strip_prefix("PT")?;
    let mut value = 0i64;
    let mut digits = String::new();
    for c in text.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else {
            let n = digits.parse::<i64>().ok()?;
            digits.clear();
            value = value.checked_add(n.checked_mul(match c {
                'H' => 60,
                'M' => 1,
                _ => return None,
            })?)?;
        }
    }
    if digits.is_empty() && !text.is_empty() && value <= 100_000 {
        Some(value)
    } else {
        None
    }
}
impl RecipeImporter for JsonLdImporter {
    fn extract(&self, input: &ImportInput) -> AppResult<ImportedRecipeDraft> {
        let v: Value = serde_json::from_str(&input.payload).map_err(|_| error())?;
        let mut recipes = vec![];
        recipe_nodes(&v, 0, &mut recipes)?;
        if recipes.len() != 1 {
            return Err(error());
        }
        let r = recipes[0];
        let ingredients = r["recipeIngredient"]
            .as_array()
            .ok_or_else(error)?
            .iter()
            .map(|v| v.as_str().map(str::to_owned).ok_or_else(error))
            .collect::<AppResult<Vec<_>>>()?;
        let yield_text = match &r["recipeYield"] {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Array(a) => a.first().map(string).unwrap_or_default(),
            _ => String::new(),
        };
        let author = match &r["author"] {
            Value::String(s) => s.clone(),
            Value::Object(o) => o.get("name").map(string).unwrap_or_default(),
            Value::Array(a) => a
                .iter()
                .map(|v| string(&v["name"]))
                .collect::<Vec<_>>()
                .join(", "),
            _ => String::new(),
        };
        let mut result = draft(
            input,
            string(&r["name"]),
            yield_text,
            ingredients,
            instructions(&r["recipeInstructions"], 0)?.join("\n\n"),
            string(&r["description"]),
            minutes(&r["prepTime"]),
            minutes(&r["cookTime"]),
            author,
        )?;
        for key in ["prepTime", "cookTime"] {
            if !r[key].is_null() && minutes(&r[key]).is_none() {
                result.warnings.push(format!(
                    "{key} could not be interpreted; check the cooking times."
                ));
            }
        }
        Ok(result)
    }
}
pub fn line(raw: &str) -> AppResult<IngredientDraft> {
    line_with_mode(raw, &crate::domain::measurements::Mode::Mixed)
}
pub(super) fn line_with_mode(
    raw: &str,
    mode: &crate::domain::measurements::Mode,
) -> AppResult<IngredientDraft> {
    bounded_text(raw, 2000)?;
    let words: Vec<&str> = raw.split_whitespace().collect();
    let mut amount = String::new();
    let mut consumed = 0;
    for count in (1..=words.len().min(2)).rev() {
        let candidate = words[..count].join(" ");
        if rational::parse(&candidate).is_ok_and(|v| v.is_some()) {
            amount = candidate;
            consumed = count;
            break;
        }
    }
    let rest = words[consumed..].join(" ");
    let lower = rest.to_lowercase();
    let (unit, convention, mut name) = match super::units::prefix(&rest, mode) {
        Some((id, region, consumed)) => {
            (Some(id), region, rest[consumed..].trim_start().to_owned())
        }
        None => (Some("piece".into()), None, rest.clone()),
    };
    if amount.is_empty() && lower.ends_with(" to taste") {
        amount = "to taste".into();
        name = rest[..rest.len() - 9].into();
    }
    Ok(IngredientDraft {
        id: Uuid::new_v4().to_string(),
        ingredient_id: None,
        amount,
        unit_id: unit,
        raw_text: raw.into(),
        source_convention: convention,
        note: format!("Review ingredient: {name}"),
        optional: false,
    })
}
#[allow(clippy::too_many_arguments)]
pub fn draft(
    input: &ImportInput,
    title: String,
    yield_text: String,
    ingredients: Vec<String>,
    instruction_text: String,
    description: String,
    prep_minutes: Option<i64>,
    cook_minutes: Option<i64>,
    attribution: String,
) -> AppResult<ImportedRecipeDraft> {
    validation::text(&title, 200, true)?;
    bounded_text(&yield_text, 200)?;
    bounded_text(&description, 10_000)?;
    bounded_text(&instruction_text, 100_000)?;
    bounded_text(&attribution, 2000)?;
    if ingredients.len() > 300 {
        return Err(error());
    }
    let servings = if rational::parse(&yield_text).is_ok_and(|v| v.is_some_and(|r| r.n > 0)) {
        yield_text.clone()
    } else {
        String::new()
    };
    let source = url::validate(&input.source_url)?;
    let meal = MealDraft {
        title,
        description,
        servings,
        prep_minutes,
        cook_minutes,
        instructions: instruction_text,
        notes: String::new(),
        source_url: Some(source.to_string()),
        source_site: Some(source.host_str().unwrap().into()),
        tags: vec![],
        ingredients: ingredients
            .iter()
            .map(|s| line(s))
            .collect::<AppResult<_>>()?,
        custom_fields: vec![],
    };
    Ok(ImportedRecipeDraft{meal,warnings:vec!["Review servings, every quantity, regional unit and ingredient identity before saving. Unresolved or missing units default to Piece; review these against the original wording. Unknown quantities remain shopping notes. Imported markup is displayed as inert text.".into()],original_yield:yield_text,attribution,duplicate_titles:vec![]})
}
