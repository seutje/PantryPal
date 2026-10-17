//! Cook Smarts' visible name-first lines; source wording is never replaced.
use super::*;
use crate::domain::{meal_library::IngredientDraft, measurements::Mode};

pub(super) fn version_query(url: &tauri::Url) -> bool {
    let recipe_path = url
        .path()
        .strip_prefix("/days/")
        .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        || url.path().strip_prefix("/recipes/").is_some_and(|slug| {
            slug.starts_with(|c: char| c.is_ascii_digit())
                && slug.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        });
    url.host_str() == Some("mealplans.cooksmarts.com")
        && recipe_path
        && matches!(
            url.query(),
            Some("version=standard" | "version=gf" | "version=paleo" | "version=vegetarian")
        )
}

pub(super) fn line(raw: &str, mode: &Mode) -> AppResult<IngredientDraft> {
    bounded_text(raw, 2000)?;
    let Some((name, quantity)) = raw.split_once(" - ") else {
        return json_ld::line_with_mode(raw, mode);
    };
    let (quantity, preparation) = quantity.split_once(',').unwrap_or((quantity, ""));
    let quantity = quantity.split(" (").next().unwrap_or(quantity).trim();
    let mut result = json_ld::line_with_mode(&format!("{quantity} {name}"), mode)?;
    if matches!(quantity.to_lowercase().as_str(), "to taste" | "as needed") {
        result.amount = quantity.into();
    }
    // Never interpret the first number of a divided amount or range as its total.
    // These source forms need a deliberate review, rather than guessed arithmetic.
    if quantity.contains(['+', '-', '–', '—']) || quantity.contains(" to ") {
        result.amount.clear();
        result.unit_id = Some("piece".into());
        result.source_convention = None;
    }
    result.raw_text = raw.into();
    result.optional = name.contains("(opt)");
    result.note = format!("Review ingredient: {name}");
    if !preparation.trim().is_empty() {
        result
            .note
            .push_str(&format!("\nPreparation: {}", preparation.trim()));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{imports::ConfirmImport, Database};

    fn input() -> ImportInput {
        ImportInput {
            source_url: "https://mealplans.cooksmarts.com/days/2590?version=standard".into(),
            format: ImportFormat::Website,
            payload: include_str!("../../../tests/fixtures/site-adapters/cook_smarts.json").into(),
            permitted: true,
        }
    }

    #[test]
    fn only_public_recipe_versions_are_allowed_and_capture_identity_stays_exact() {
        for version in ["standard", "gf", "paleo", "vegetarian"] {
            assert!(url::validate(&format!(
                "https://mealplans.cooksmarts.com/days/2590?version={version}"
            ))
            .is_ok());
            assert!(sites::capture_script(&format!(
                "https://mealplans.cooksmarts.com/recipes/2590-fixture?version={version}"
            ))
            .is_ok());
        }
        for url in [
            "https://mealplans.cooksmarts.com/days/2590?version=standard&token=secret",
            "https://mealplans.cooksmarts.com/days/2590?version=standard&version=gf",
            "https://mealplans.cooksmarts.com/days/2590?version=unknown",
            "https://mealplans.cooksmarts.com/days/toggle_metric?show=true",
            "https://mealplans.cooksmarts.com/login?version=standard",
            "https://mealplans.cooksmarts.com.evil.org/days/2590?version=standard",
            "https://example.com/days/2590?version=standard",
        ] {
            assert!(url::validate(url).is_err());
        }
        let a = url::validate(&input().source_url).unwrap();
        let b = url::validate("https://mealplans.cooksmarts.com/days/2590?version=gf").unwrap();
        assert!(!sites::same_page(&a, &b));
        let mut source = input();
        source.permitted = false;
        assert!(extract(&source).is_err());
        source.permitted = true;
        let mut payload: serde_json::Value = serde_json::from_str(&source.payload).unwrap();
        payload["sections"] = serde_json::json!([]);
        source.payload = payload.to_string();
        assert!(extract(&source).is_err());
        assert!(sites::site("attacker.cooksmarts.com").is_none());
    }

    #[tokio::test]
    async fn preview_modes_and_save_reopen_preserve_name_first_lines_sections_and_unknowns() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path()).await.unwrap();
        let source = input();
        for (mode, region) in [
            (Mode::Mixed, "us"),
            (Mode::Us, "us"),
            (Mode::Metric, "metric"),
        ] {
            let mut preferences = db.measurement_preferences().await.unwrap();
            preferences.mode = mode;
            db.save_measurement_preferences(preferences).await.unwrap();
            let preview = db.preview_import(&source).await.unwrap();
            let lines = &preview.meal.ingredients;
            assert_eq!(preview.meal.servings, "4");
            assert_eq!(lines[0].amount, "2");
            assert_eq!(lines[1].unit_id.as_deref(), Some("clove"));
            assert_eq!(lines[1].amount, "4");
            assert_eq!(lines[2].amount, "1 1/2");
            assert_eq!(
                lines[2].unit_id.as_deref(),
                Some(format!("tsp-{region}").as_str())
            );
            assert!(lines[3].optional);
            assert_eq!(lines[4].unit_id.as_deref(), Some("lb"));
            assert_eq!(lines[5].unit_id.as_deref(), Some("oz"));
            assert_eq!(lines[6].amount, "");
            assert_eq!(lines[6].raw_text, "Oil, olive - 1 Tbsp + 2 Tbsp");
            assert_eq!(
                lines[7].unit_id.as_deref(),
                Some(format!("cup-{region}").as_str())
            );
            assert_eq!(lines[8].amount, "to taste");
            assert!(lines[0].note.contains("for 2 nights"));
            assert!(lines[1].note.contains("Preparation: chopped"));
            assert!(preview.meal.instructions.starts_with("Prep\n\n"));
            assert!(preview.meal.instructions.contains("\n\nMake\n\n"));
            assert_eq!(preview.attribution, "Cook Smarts");
            assert!(preview.meal.notes.contains("Active: 60"));
            assert!(preview.meal.prep_minutes.is_none());
            assert!(preview.meal.cook_minutes.is_none());
        }
        assert_eq!(db.status().await.unwrap().meal_count, 0);
        let preview = db.preview_import(&source).await.unwrap();
        let original = preview.meal.clone();
        let saved = db
            .confirm_import(ConfirmImport {
                action_id: "cook-smarts-fixture".into(),
                source,
                reviewed: preview.meal,
                confirmed: true,
                allow_duplicate: false,
            })
            .await
            .unwrap();
        db.close().await;
        let reopened = Database::open(dir.path()).await.unwrap();
        let loaded = reopened.meal(&saved.id).await.unwrap();
        assert_eq!(
            loaded.data.ingredients[0].raw_text,
            original.ingredients[0].raw_text
        );
        assert_eq!(
            loaded.data.ingredients[0].note,
            original.ingredients[0].note
        );
        assert!(loaded.data.ingredients[6].quantity.is_none());
        assert!(loaded.data.ingredients[3].optional);
        assert_eq!(loaded.data.source_url, original.source_url);
        reopened.close().await;
    }

    #[test]
    fn ranges_unknown_units_and_missing_amounts_stay_reviewable() {
        for raw in [
            "Onions - 2-3",
            "Onions - 2 to 3",
            "Oil - 1 Tbsp + 2 Tbsp",
            "Salt - as needed",
            "Water",
        ] {
            let parsed = line(raw, &Mode::Mixed).unwrap();
            assert!(parsed.amount.is_empty() || parsed.amount == "as needed");
            assert_eq!(parsed.raw_text, raw);
        }
        assert_eq!(
            line("Shallots - 1 bulb, chopped", &Mode::Mixed)
                .unwrap()
                .unit_id
                .as_deref(),
            Some("piece")
        );
    }
}
