use super::*;
use serde_json::json;
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
    assert!(draft.meal.ingredients[3].unit_id.is_none());
    assert_eq!(draft.meal.ingredients[4].amount, "");
    assert_eq!(draft.meal.ingredients[5].unit_id.as_deref(), Some("can"));
    assert!(draft.meal.ingredients[7].unit_id.is_none());
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
