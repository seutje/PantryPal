//! Budget Bytes appends grocery prices to its visible WPRM ingredient lines.
use super::*;
use crate::domain::{meal_library::IngredientDraft, measurements::Mode};

pub(super) fn line(raw: &str, mode: &Mode) -> AppResult<IngredientDraft> {
    bounded_text(raw, 2000)?;
    // Remove only an explicit trailing dollar price from the matching name.
    // Preparation, package sizes and other parenthetical wording stay intact.
    if let Some((text, price)) = raw.trim_end().rsplit_once(" ($") {
        if let Some(value) = price.strip_suffix(')') {
            if let Some((dollars, cents)) = value.split_once('.') {
                if !dollars.is_empty()
                    && dollars.bytes().all(|c| c.is_ascii_digit())
                    && cents.len() == 2
                    && cents.bytes().all(|c| c.is_ascii_digit())
                {
                    let mut result = json_ld::line_with_mode(text, mode)?;
                    result.raw_text = raw.into();
                    result.note.push_str(&format!("\nSource price: (${value})"));
                    return Ok(result);
                }
            }
        }
    }
    json_ld::line_with_mode(raw, mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{imports::ConfirmImport, Database};

    #[test]
    fn only_trailing_prices_are_separated_from_ingredient_names() {
        for raw in [
            "2 Tbsp olive oil (extra virgin)",
            "1 (400 g) can beans ($1.00 per can)",
            "2 Tbsp olive oil ($unknown)",
            "2 Tbsp olive oil ($0.32) divided",
        ] {
            let parsed = line(raw, &Mode::Mixed).unwrap();
            assert_eq!(parsed.note, json_ld::line(raw).unwrap().note);
            assert_eq!(parsed.raw_text, raw);
        }
        let parsed = line("olive oil to taste ($0.32)", &Mode::Mixed).unwrap();
        assert_eq!(parsed.amount, "to taste");
        assert_eq!(
            parsed.note,
            "Review ingredient: olive oil\nSource price: ($0.32)"
        );
    }

    #[tokio::test]
    async fn website_prices_do_not_block_exact_matches_and_survive_save_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path()).await.unwrap();
        let source = ImportInput {
            source_url: "https://www.budgetbytes.com/sheet-pan-kielbasa-potatoes-and-green-beans/"
                .into(),
            format: ImportFormat::Website,
            payload: serde_json::json!({
                "version": 1, "site": "budget_bytes",
                "recipe": {"@type": "Recipe", "name": "Price matching fixture",
                    "recipeYield": "4", "recipeIngredient": ["2 Tbsp olive oil ($0.32)"],
                    "recipeInstructions": ["Original fixture instruction."]}
            })
            .to_string(),
            permitted: true,
        };
        let olive = db
            .catalog()
            .await
            .unwrap()
            .ingredients
            .into_iter()
            .find(|i| i.data.name.to_lowercase() == "olive oil")
            .unwrap();
        for mode in [Mode::Mixed, Mode::Us, Mode::Metric] {
            let mut preferences = db.measurement_preferences().await.unwrap();
            preferences.mode = mode.clone();
            db.save_measurement_preferences(preferences).await.unwrap();
            let draft = db.preview_import(&source).await.unwrap();
            let parsed = &draft.meal.ingredients[0];
            assert_eq!(parsed.ingredient_id.as_deref(), Some(olive.id.as_str()));
            assert_eq!(parsed.amount, "2");
            assert_eq!(
                parsed.unit_id.as_deref(),
                Some(if mode == Mode::Metric {
                    "tbsp-metric"
                } else {
                    "tbsp-us"
                })
            );
            assert_eq!(parsed.raw_text, "2 Tbsp olive oil ($0.32)");
            assert_eq!(parsed.note, "Source price: ($0.32)");
        }
        let reviewed = db.preview_import(&source).await.unwrap().meal;
        let saved = db
            .confirm_import(ConfirmImport {
                action_id: "budget-price-fixture".into(),
                source,
                reviewed,
                confirmed: true,
                allow_duplicate: false,
            })
            .await
            .unwrap();
        db.close().await;
        let reopened = Database::open(dir.path()).await.unwrap();
        let loaded = reopened.meal(&saved.id).await.unwrap();
        assert_eq!(
            loaded.data.ingredients[0].ingredient_id.as_deref(),
            Some(olive.id.as_str())
        );
        assert_eq!(
            loaded.data.ingredients[0].raw_text,
            "2 Tbsp olive oil ($0.32)"
        );
        assert_eq!(loaded.data.ingredients[0].note, "Source price: ($0.32)");
        reopened.close().await;
    }
}
