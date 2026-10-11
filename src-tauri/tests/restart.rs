use pantrypal_lib::{
    domain::{
        catalog::IngredientInput,
        meals::MealInput,
        measurements::{CupConvention, Mode, RuleInput},
    },
    persistence::Database,
};
use std::{path::PathBuf, process::Command};

#[test]
fn crud_survives_real_process_restart() {
    let directory = tempfile::tempdir().unwrap();
    for mode in ["write", "read"] {
        let result = Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "restart_child", "--nocapture"])
            .env("PANTRYPAL_RESTART_TEST_DIR", directory.path())
            .env("PANTRYPAL_RESTART_TEST_MODE", mode)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "child {mode} failed: {} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[tokio::test]
#[ignore = "executed by the parent in two separate processes with an isolated database"]
async fn restart_child() {
    let directory = PathBuf::from(std::env::var_os("PANTRYPAL_RESTART_TEST_DIR").unwrap());
    let mode = std::env::var("PANTRYPAL_RESTART_TEST_MODE").unwrap();
    let db = Database::open(&directory).await.unwrap();
    if mode == "write" {
        let ingredient = db
            .create_ingredient(IngredientInput {
                name: "Restart herb".into(),
                category_id: None,
                section_id: None,
                preferred_unit_id: Some("g".into()),
                aliases: vec!["Restart leaves".into()],
                season_months: vec![6, 7],
                notes: "Initial note".into(),
            })
            .await
            .unwrap();
        let mut input = ingredient.data;
        input.notes = "Persisted edit".into();
        db.update_ingredient(&ingredient.id, input).await.unwrap();
        let mut preferences = db.measurement_preferences().await.unwrap();
        preferences.mode = Mode::Metric;
        preferences.cup_convention = Some(CupConvention::Us);
        preferences.fractions = false;
        preferences.decimal_places = 2;
        db.save_measurement_preferences(preferences).await.unwrap();
        db.save_conversion_rule(
            None,
            RuleInput {
                ingredient_id: ingredient.id.clone(),
                from_unit_id: "piece".into(),
                from_amount: "½".into(),
                to_unit_id: "g".into(),
                to_amount: "75".into(),
                approximate: true,
                notes: "Medium bundle".into(),
            },
        )
        .await
        .unwrap();
        db.retire_ingredient("ingredient-garlic", true)
            .await
            .unwrap();
        let mut meal: MealInput =
            serde_json::from_str(include_str!("../../tests/fixtures/demo-meal.json")).unwrap();
        meal.base_servings_text = Some("4".into());
        meal.ingredients[0].amount_text = Some("⅓".into());
        meal.ingredients[2].amount_text = Some("to taste".into());
        let definition = db.create_field(serde_json::from_value(serde_json::json!({"label":"Freezes well","kind":"boolean","description":"Household preference","options":[],"minimum":null,"maximum":null,"unitLabel":null,"sortOrder":0})).unwrap()).await.unwrap();
        meal.custom_fields.push(serde_json::from_value(serde_json::json!({"fieldId":definition.id,"value":{"kind":"boolean","value":false}})).unwrap());
        let created = db.create_meal(meal).await.unwrap();
        let mut input = created.data;
        input.title = "Restart supper".into();
        db.update_meal(&created.id, 1, input).await.unwrap();
        let mut week = db.get_week("2021-01-01").await.unwrap();
        for (index, day) in [(0, None), (1, Some("2021-01-01"))] {
            week=db.apply_plan(serde_json::from_value(serde_json::json!({"actionId":format!("restart-plan-{index}"),"weekStart":week.week_start,"expectedRevision":week.revision,"operation":{"kind":"add","entryId":format!("restart-entry-{index}"),"mealId":created.id,"localDate":day,"servings":"1 ½"}})).unwrap()).await.unwrap();
        }
        db.apply_plan(serde_json::from_value(serde_json::json!({"actionId":"restart-cook","weekStart":week.week_start,"expectedRevision":week.revision,"operation":{"kind":"set_status","entryId":"restart-entry-1","status":"cooked","cookedDate":"2021-01-01","confirmed":false}})).unwrap()).await.unwrap();
    } else {
        let catalog = db.catalog().await.unwrap();
        let ingredient = catalog
            .ingredients
            .iter()
            .find(|item| item.data.name == "Restart herb")
            .unwrap();
        assert_eq!(ingredient.data.notes, "Persisted edit");
        assert_eq!(ingredient.data.aliases, ["Restart leaves"]);
        assert_eq!(ingredient.data.season_months, [6, 7]);
        let preferences = db.measurement_preferences().await.unwrap();
        assert_eq!(preferences.mode, Mode::Metric);
        assert_eq!(preferences.cup_convention, Some(CupConvention::Us));
        assert!(!preferences.fractions);
        assert_eq!(preferences.decimal_places, 2);
        let rules = db.conversion_rules(&ingredient.id).await.unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].original_from_text, "½");
        assert!(rules[0].approximate);
        assert_eq!(rules[0].notes, "Medium bundle");
        assert!(db.ingredient("ingredient-garlic").await.unwrap().retired);
        let meals = db.list_meals().await.unwrap();
        assert_eq!(meals.len(), 1);
        assert_eq!(meals[0].data.title, "Restart supper");
        assert_eq!(meals[0].revision, 2);
        assert_eq!(meals[0].data.ingredients[0].raw_text, "1/3 US cup rice");
        assert!(meals[0].data.ingredients[2].quantity.is_none());
        let fields = db.list_fields().await.unwrap();
        let input = serde_json::from_value(serde_json::json!({"text":"rice","ingredientId":"ingredient-broccoli","tag":"simple","field":{"fieldId":fields[0].id,"operator":"equals","value":{"kind":"boolean","value":false}},"cooked":"cooked","cookedFrom":null,"cookedTo":null})).unwrap();
        let library = db.search_meals(input).await.unwrap();
        assert_eq!(library.meals.len(), 1);
        assert_eq!(
            library.meals[0].meal.data.ingredients[0]
                .amount_text
                .as_deref(),
            Some("⅓")
        );
        let mut input = library.meals[0].meal.data.clone();
        input.notes = "Edited after process restart".into();
        let edited = db
            .update_meal(&meals[0].id, 2, input.clone())
            .await
            .unwrap();
        assert_eq!(edited.data, input);
        assert_eq!(edited.revision, 3);
        let week = db.get_week("2021-01-01").await.unwrap();
        assert_eq!(week.week_start, "2020-12-28");
        assert_eq!(week.entries.len(), 2);
        assert_eq!(week.entries[0].servings_text, "1 ½");
        assert_eq!(db.meal_history(&meals[0].id).await.unwrap().events.len(), 1);
        db.apply_plan(serde_json::from_value(serde_json::json!({"actionId":"restart-cook","weekStart":week.week_start,"expectedRevision":2,"operation":{"kind":"set_status","entryId":"restart-entry-1","status":"cooked","cookedDate":"2021-01-01","confirmed":false}})).unwrap()).await.unwrap();
        assert_eq!(
            db.meal_history(&meals[0].id)
                .await
                .unwrap()
                .last_cooked
                .as_deref(),
            Some("2021-01-01")
        );
        assert_eq!(db.status().await.unwrap().schema_version, 7);
    }
    db.close().await;
}
