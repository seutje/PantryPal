//! Publisher adapters requested by the owner; only visible recipe content is read.
//! The native host reads a bounded visible recipe; this module validates drafts.
use super::*;
use serde_json::Value;

pub struct WebsiteImporter;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Capture {
    version: u32,
    site: String,
    recipe: Value,
    #[serde(default)]
    sections: Vec<String>,
}
pub fn site(host: &str) -> Option<&'static str> {
    match host {
        "budgetbytes.com" | "www.budgetbytes.com" => Some("budget_bytes"),
        "allrecipes.com" | "www.allrecipes.com" => Some("allrecipes"),
        "spendwithpennies.com" | "www.spendwithpennies.com" => Some("spend_with_pennies"),
        "recipetineats.com" | "www.recipetineats.com" => Some("recipe_tin_eats"),
        "cooking.nytimes.com" => Some("nyt_cooking"),
        "mealplans.cooksmarts.com" => Some("cook_smarts"),
        _ => None,
    }
}
pub fn error() -> AppError {
    AppError::validation("The visible recipe could not be read safely. Finish any website login or access prompt, wait for the recipe to load and try again, or use manual entry.")
}
pub fn capture_script(source_url: &str) -> AppResult<String> {
    let url = url::validate(source_url)?;
    let site = site(url.host_str().unwrap()).ok_or_else(|| {
        AppError::validation(
            "No authorized adapter is available for this website. Use manual entry.",
        )
    })?;
    let spec = serde_json::json!({"site":site,"url":url.as_str()});
    Ok(format!("({})({spec})", include_str!("capture.js")))
}
pub fn same_page(a: &tauri::Url, b: &tauri::Url) -> bool {
    a.origin() == b.origin()
        && a.path().trim_end_matches('/') == b.path().trim_end_matches('/')
        && a.query() == b.query()
        && a.fragment() == b.fragment()
}
impl RecipeImporter for WebsiteImporter {
    fn extract(&self, input: &ImportInput) -> AppResult<ImportedRecipeDraft> {
        let url = url::validate(&input.source_url)?;
        let expected = site(url.host_str().unwrap()).ok_or_else(error)?;
        let capture: Capture = serde_json::from_str(&input.payload).map_err(|_| error())?;
        if capture.version != 1 || capture.site != expected {
            return Err(error());
        }
        // A partial/gated recipe is never a successful automatic import.
        if !capture.recipe["recipeIngredient"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
            || !capture.recipe["recipeInstructions"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        {
            return Err(error());
        }
        let mut projected = input.clone();
        projected.format = ImportFormat::JsonLd;
        projected.payload = serde_json::to_string(&capture.recipe).map_err(|_| error())?;
        let mut draft = json_ld::JsonLdImporter.extract(&projected)?;
        if expected == "cook_smarts" {
            if capture.sections.len() != draft.meal.ingredients.len() {
                return Err(error());
            }
            for (line, section) in draft.meal.ingredients.iter_mut().zip(&capture.sections) {
                bounded_text(section, 200)?;
                *line =
                    cook_smarts::line(&line.raw_text, &crate::domain::measurements::Mode::Mixed)?;
                if !section.is_empty() {
                    line.note.push_str(&format!("\nSection: {section}"));
                }
                bounded_text(&line.note, 2000)?;
            }
            draft.warnings.push("Cook Smarts: check the selected diet, servings, multi-night sections and divided quantities. Compound/range quantities remain unknown for explicit review. Active/Total times are retained in notes, not interpreted as prep/cook times.".into());
            if let Some(notes) = capture.recipe["description"].as_str() {
                draft.meal.notes = notes.into();
            }
        } else {
            if !capture.sections.is_empty() {
                return Err(error());
            }
            if expected == "budget_bytes" {
                for line in &mut draft.meal.ingredients {
                    *line = budget_bytes::line(
                        &line.raw_text,
                        &crate::domain::measurements::Mode::Mixed,
                    )?;
                }
            }
        }
        draft.warnings.push("Read from the visible website recipe. Check ingredient sections, optional items and any website serving/unit adjustments; review the resolved regional units before saving.".into());
        Ok(draft)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_five_adapter_fixtures_keep_raw_text_and_reject_mismatched_provenance() {
        let fixtures = [
            (
                "www.budgetbytes.com",
                include_str!("../../../tests/fixtures/site-adapters/budget_bytes.json"),
            ),
            (
                "www.allrecipes.com",
                include_str!("../../../tests/fixtures/site-adapters/allrecipes.json"),
            ),
            (
                "www.spendwithpennies.com",
                include_str!("../../../tests/fixtures/site-adapters/spend_with_pennies.json"),
            ),
            (
                "www.recipetineats.com",
                include_str!("../../../tests/fixtures/site-adapters/recipe_tin_eats.json"),
            ),
            (
                "cooking.nytimes.com",
                include_str!("../../../tests/fixtures/site-adapters/nyt_cooking.json"),
            ),
        ];
        for (host, payload) in fixtures {
            let mut input = ImportInput {
                source_url: format!("https://{host}/fixture"),
                format: ImportFormat::Website,
                payload: payload.into(),
                permitted: true,
            };
            let draft = extract(&input).unwrap();
            assert_eq!(draft.meal.ingredients[0].raw_text, "½ cup milk");
            assert_eq!(draft.meal.ingredients[0].amount, "½");
            assert_eq!(draft.meal.ingredients[0].unit_id.as_deref(), Some("cup-us"));
            assert_eq!(draft.meal.ingredients[1].amount, "to taste");
            assert_eq!(draft.attribution, "PantryPal fixture author");
            assert_eq!(draft.meal.prep_minutes, Some(5));
            input.permitted = false;
            assert!(extract(&input).is_err());
            input.permitted = true;
            input.source_url = "https://example.com/fixture".into();
            assert!(extract(&input).is_err());
        }
    }
    #[test]
    fn capture_policy_rejects_spoofed_hosts_incomplete_content_and_versions() {
        assert!(site("www.budgetbytes.com.attacker.org").is_none());
        assert!(site("attacker.budgetbytes.com").is_none());
        assert!(site("www.nytimes.com").is_none());
        assert!(capture_script("https://cooksmarts.com/fixture").is_err());
        let payload: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/site-adapters/budget_bytes.json"
        ))
        .unwrap();
        for (field, value) in [
            ("version", serde_json::json!(2)),
            ("site", serde_json::json!("nyt_cooking")),
        ] {
            let mut changed = payload.clone();
            changed[field] = value;
            let input = ImportInput {
                source_url: "https://www.budgetbytes.com/fixture".into(),
                format: ImportFormat::Website,
                payload: changed.to_string(),
                permitted: true,
            };
            assert!(extract(&input).is_err());
        }
        let a = url::validate("https://www.budgetbytes.com/fixture/").unwrap();
        let b = url::validate("https://www.budgetbytes.com/fixture").unwrap();
        assert!(same_page(&a, &b));
        assert!(!same_page(
            &a,
            &url::validate("https://www.budgetbytes.com/different").unwrap()
        ));
    }
}
