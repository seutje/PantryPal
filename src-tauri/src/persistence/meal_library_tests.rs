use super::Database;
use crate::domain::{
    meal_library::{MealDraft, MealSearch},
    meals::MealInput,
};
use serde_json::json;
fn search() -> MealSearch {
    serde_json::from_value(json!({"text":"","ingredientId":null,"tag":"","field":null,"cooked":"any","cookedFrom":null,"cookedTo":null})).unwrap()
}
#[tokio::test]
async fn cooked_filters_use_actual_events_and_validate_local_dates() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let meal = db
        .create_meal(
            serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap(),
        )
        .await
        .unwrap();
    sqlx::query("INSERT INTO plan_entry(id,week_start,local_date,meal_id,servings_numerator,servings_denominator) VALUES ('scheduled','2026-10-12','2026-10-13',?,4,1)").bind(&meal.id).execute(&db.pool).await.unwrap();
    assert!(db.search_meals(search()).await.unwrap().meals[0]
        .last_cooked
        .is_none());
    sqlx::query("INSERT INTO meal_cooked_event(id,meal_id,local_date) VALUES ('first',?,'2026-10-01'),('second',?,'2026-10-09')").bind(&meal.id).bind(&meal.id).execute(&db.pool).await.unwrap();
    assert_eq!(
        db.search_meals(search()).await.unwrap().meals[0]
            .last_cooked
            .as_deref(),
        Some("2026-10-09")
    );
    let mut filter = search();
    filter.cooked = crate::domain::meal_library::CookedFilter::Never;
    assert!(db.search_meals(filter).await.unwrap().meals.is_empty());
    let mut filter = search();
    filter.cooked_from = Some("2026-10-09".into());
    filter.cooked_to = Some("2026-10-09".into());
    assert_eq!(db.search_meals(filter).await.unwrap().meals.len(), 1);
    let mut filter = search();
    filter.cooked_to = Some("2026-10-08".into());
    assert!(db.search_meals(filter).await.unwrap().meals.is_empty());
    for invalid in ["2026-02-30", "2026-1-01", "invalid"] {
        let mut filter = search();
        filter.cooked_from = Some(invalid.into());
        assert!(db.search_meals(filter).await.is_err());
    }
    let mut filter = search();
    filter.cooked_from = Some("2026-10-10".into());
    filter.cooked_to = Some("2026-10-09".into());
    assert!(db.search_meals(filter).await.is_err());
    db.close().await;
}
#[tokio::test]
async fn unchanged_drafts_preserve_legacy_unreduced_pairs_and_nulls() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let mut input: MealInput =
        serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap();
    input.base_servings.numerator = 8;
    input.base_servings.denominator = 2;
    input.ingredients[0].quantity.as_mut().unwrap().numerator = 2;
    input.ingredients[0].quantity.as_mut().unwrap().denominator = 6;
    let old = db.create_meal(input).await.unwrap();
    let draft:MealDraft=serde_json::from_value(json!({"title":old.data.title,"description":old.data.description,"servings":"8/2","prepMinutes":old.data.prep_minutes,"cookMinutes":old.data.cook_minutes,"instructions":old.data.instructions,"notes":"Edited","sourceUrl":null,"sourceSite":null,"tags":old.data.tags,"ingredients":old.data.ingredients.iter().map(|l|json!({"id":l.id,"ingredientId":l.ingredient_id,"amount":l.quantity.as_ref().map(|q|format!("{}/{}",q.numerator,q.denominator)).unwrap_or_default(),"unitId":l.unit_id,"rawText":l.raw_text,"sourceConvention":l.source_convention,"note":l.note,"optional":l.optional})).collect::<Vec<_>>(),"customFields":[]})).unwrap();
    // Integer amounts use the legacy editor's exact integer spelling.
    let mut draft = draft;
    draft.ingredients[1].amount = "500".into();
    let edited = db
        .save_meal_draft(Some(old.id), Some(1), draft)
        .await
        .unwrap();
    assert_eq!(edited.data.base_servings, old.data.base_servings);
    assert_eq!(
        edited.data.ingredients[0].quantity,
        old.data.ingredients[0].quantity
    );
    assert!(edited.data.ingredients[2].quantity.is_none());
    assert_eq!(
        edited.data.ingredients[0].amount_text.as_deref(),
        Some("2/6")
    );
    let mut invalid = edited.data.clone();
    invalid.ingredients[0].amount_text = Some("9".into());
    assert!(db.update_meal(&edited.id, 2, invalid).await.is_err());
    assert_eq!(db.meal(&edited.id).await.unwrap(), edited);
    db.close().await;
}
