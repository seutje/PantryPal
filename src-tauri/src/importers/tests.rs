use super::*;
use crate::domain::measurements::Mode;

#[test]
fn unresolved_import_units_default_to_piece_without_inventing_amounts_or_losing_text() {
    for mode in [Mode::Mixed, Mode::Us, Mode::Metric] {
        for (raw, amount, name) in [
            ("2 onions", "2", "onions"),
            ("1 scoop flour", "1", "scoop flour"),
            ("1 (400 g) can beans", "1", "(400 g) can beans"),
            ("salt to taste", "to taste", "salt"),
            ("pepper", "", "pepper"),
            ("2-3 onions", "", "2-3 onions"),
        ] {
            let line = parse_line(raw, &mode).unwrap();
            assert_eq!(line.unit_id.as_deref(), Some("piece"));
            assert_eq!(line.amount, amount);
            assert_eq!(line.raw_text, raw);
            assert_eq!(line.note, format!("Review ingredient: {name}"));
            assert!(line.source_convention.is_none());
            assert!(line.ingredient_id.is_none());
        }
    }
}
use serde_json::json;

#[test]
fn import_unit_aliases_respect_mode_and_explicit_source_variants() {
    for mode in [Mode::Mixed, Mode::Us, Mode::Metric] {
        let region = if mode == Mode::Metric { "metric" } else { "us" };
        for (aliases, expected) in [
            (vec!["lb", "lbs", "pound", "pounds", "LB."], "lb".to_owned()),
            (vec!["oz", "ounce", "ounces"], "oz".to_owned()),
            (vec!["cup", "cups"], format!("cup-{region}")),
            (
                vec!["tbsp", "tbsp.", "tablespoon", "tablespoons"],
                format!("tbsp-{region}"),
            ),
            (
                vec!["tsp", "tsp.", "teaspoon", "teaspoons"],
                format!("tsp-{region}"),
            ),
            (
                vec!["fl oz", "fl. oz.", "fluid ounce", "fluid ounces"],
                "floz-us".into(),
            ),
            (vec!["pint", "pints", "pt"], "pint-us".into()),
            (vec!["quart", "quarts", "qt"], "quart-us".into()),
            (vec!["gallon", "gallons", "gal"], "gallon-us".into()),
            (vec!["gram", "grams", "g"], "g".into()),
            (vec!["millilitre", "milliliters", "mL"], "ml".into()),
            (vec!["clove", "cloves"], "clove".into()),
            (vec!["package", "packages", "pkg"], "package".into()),
        ] {
            for alias in aliases {
                let raw = format!("1 1/2 {alias} test ingredient");
                let line = parse_line(&raw, &mode).unwrap();
                assert_eq!(
                    line.unit_id.as_deref(),
                    Some(expected.as_str()),
                    "{raw} {mode:?}"
                );
                assert_eq!(line.amount, "1 1/2");
                assert_eq!(line.raw_text, raw);
                assert_eq!(line.note, "Review ingredient: test ingredient");
            }
        }
        for (raw, id, region) in [
            ("1 Imperial cups milk", "cup-imperial", "imperial"),
            ("1 US tablespoon oil", "tbsp-us", "us"),
            ("1 metric teaspoons salt", "tsp-metric", "metric"),
            ("1 Imperial fluid ounce milk", "floz-imperial", "imperial"),
        ] {
            let line = parse_line(raw, &mode).unwrap();
            assert_eq!(line.unit_id.as_deref(), Some(id));
            assert_eq!(line.source_convention.as_deref(), Some(region));
        }
        for raw in [
            "1 ground beef",
            "1 cupful milk",
            "1 (400 g) can beans",
            "1 large onion",
        ] {
            assert_eq!(
                parse_line(raw, &mode).unwrap().unit_id.as_deref(),
                Some("piece")
            );
        }
    }
}

#[tokio::test]
async fn philly_cheesesteak_website_preview_uses_persisted_mode_without_changing_amounts() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = crate::persistence::Database::open(dir.path())
        .await
        .unwrap();
    let input = ImportInput {
        source_url: "https://www.spendwithpennies.com/philly-cheesesteak-casserole/".into(),
        format: ImportFormat::Website,
        payload: include_str!(
            "../../../tests/fixtures/site-adapters/philly-cheesesteak-measurements.json"
        )
        .into(),
        permitted: true,
    };
    for mode in [Mode::Mixed, Mode::Us, Mode::Metric] {
        let mut prefs = db.measurement_preferences().await.unwrap();
        prefs.mode = mode.clone();
        // The former cup review preference must not override the new mode rule.
        prefs.cup_convention = Some(crate::domain::measurements::CupConvention::Imperial);
        db.save_measurement_preferences(prefs).await.unwrap();
        let draft = db.preview_import(&input).await.unwrap();
        let region = if mode == Mode::Metric { "metric" } else { "us" };
        let ids = [
            Some("oz"),
            Some("lb"),
            Some("tsp"),
            None,
            Some("oz"),
            None,
            Some("tbsp"),
            Some("tbsp"),
            Some("cup"),
            Some("cup"),
            Some("tsp"),
            None,
            Some("tsp"),
            Some("cup"),
            Some("cup"),
        ];
        assert_eq!(draft.meal.ingredients.len(), ids.len());
        for (line, id) in draft.meal.ingredients.iter().zip(ids) {
            let expected = Some(
                id.map(|id| {
                    if matches!(id, "tsp" | "tbsp" | "cup") {
                        format!("{id}-{region}")
                    } else {
                        id.into()
                    }
                })
                .unwrap_or_else(|| "piece".into()),
            );
            assert_eq!(line.unit_id, expected, "{} {mode:?}", line.raw_text);
        }
        assert_eq!(draft.meal.ingredients[8].amount, "1¾");
        assert_eq!(draft.meal.ingredients[8].raw_text, "1¾ cups beef broth");
        assert_eq!(draft.meal.ingredients[9].amount, "⅔");
        assert_eq!(draft.meal.ingredients[1].amount, "1");
        assert!(draft.warnings.iter().any(|w| w.contains("Unqualified")));
    }
    assert_eq!(db.status().await.unwrap().meal_count, 0);
    db.close().await;
}
pub fn fixture() -> ImportInput {
    ImportInput {
        source_url: "https://example.com/owned-recipe".into(),
        format: ImportFormat::JsonLd,
        payload: include_str!("../../../tests/fixtures/owned-recipe.json").into(),
        permitted: true,
    }
}
#[test]
fn owned_json_ld_keeps_raw_quantities_sections_and_ambiguities() {
    let draft = extract(&fixture()).unwrap();
    assert_eq!(draft.original_yield, "4 servings");
    assert_eq!(draft.meal.servings, "");
    assert_eq!(draft.meal.prep_minutes, Some(65));
    assert_eq!(draft.meal.ingredients[0].amount, "½");
    assert_eq!(draft.meal.ingredients[0].unit_id.as_deref(), Some("cup-us"));
    assert_eq!(draft.meal.ingredients[1].amount, "1 1/2");
    assert_eq!(draft.meal.ingredients[2].amount, "to taste");
    assert_eq!(draft.meal.ingredients[3].unit_id.as_deref(), Some("cup-us"));
    assert_eq!(draft.meal.ingredients[4].amount, "");
    assert_eq!(draft.meal.ingredients[5].unit_id.as_deref(), Some("can"));
    assert_eq!(draft.meal.ingredients[7].unit_id.as_deref(), Some("piece"));
    assert_eq!(draft.meal.ingredients[7].raw_text, "1 (400 g) can beans");
    assert_eq!(
        draft.meal.instructions,
        "Prepare\n\nCook this original fixture."
    );
    assert_eq!(draft.attribution, "PantryPal test author");
}
#[test]
fn supplied_sites_support_explicit_manual_review_without_scraping() {
    let sites: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../tests/fixtures/import-sites.json")).unwrap();
    for site in sites {
        let mut input = fixture();
        input.source_url = site["url"].as_str().unwrap().into();
        input.format = ImportFormat::Manual;
        input.payload=json!({"title":"Original manual fixture","servings":"1/2","ingredients":["salt to taste"],"instructions":"Original test instruction"}).to_string();
        assert!(url::site_message(
            url::validate(&input.source_url)
                .unwrap()
                .host_str()
                .unwrap()
        )
        .contains(site["message"].as_str().unwrap()));
        assert_eq!(extract(&input).unwrap().meal.servings, "1/2");
    }
}
#[test]
fn rejects_malformed_huge_deep_multiple_or_unpermitted_content_and_keeps_markup_inert() {
    let mut input = fixture();
    input.permitted = false;
    assert!(extract(&input).is_err());
    input.permitted = true;
    for payload in [
        "not JSON".into(),
        " ".into(),
        "x".repeat(MAX_PAYLOAD + 1),
        json!({"@type":"Recipe","name":"No ingredients"}).to_string(),
        format!("[{},{}]", input.payload, input.payload),
        format!("{}0{}", "[".repeat(130), "]".repeat(130)),
    ] {
        input.payload = payload;
        assert!(extract(&input).is_err());
    }
    input.payload=json!({"@type":"Recipe","name":"<script>inert</script>","recipeYield":4,"recipeIngredient":["<img src=x onerror=alert(1)>"],"recipeInstructions":"<b>Inert text</b>","cookTime":"P1D"}).to_string();
    let draft = extract(&input).unwrap();
    assert_eq!(draft.meal.title, "<script>inert</script>");
    assert_eq!(draft.meal.instructions, "<b>Inert text</b>");
    assert!(draft.meal.cook_minutes.is_none());
    assert!(draft.warnings.len() > 1);
}
