use pantrypal_lib::{
    domain::{catalog::IngredientInput, meals::MealInput},
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
        db.retire_ingredient("ingredient-garlic", true)
            .await
            .unwrap();
        let meal: MealInput =
            serde_json::from_str(include_str!("../../tests/fixtures/demo-meal.json")).unwrap();
        let created = db.create_meal(meal).await.unwrap();
        let mut input = created.data;
        input.title = "Restart supper".into();
        db.update_meal(&created.id, 1, input).await.unwrap();
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
        assert!(db.ingredient("ingredient-garlic").await.unwrap().retired);
        let meals = db.list_meals().await.unwrap();
        assert_eq!(meals.len(), 1);
        assert_eq!(meals[0].data.title, "Restart supper");
        assert_eq!(meals[0].revision, 2);
        assert_eq!(meals[0].data.ingredients[0].raw_text, "1/3 US cup rice");
        assert!(meals[0].data.ingredients[2].quantity.is_none());
        assert_eq!(db.status().await.unwrap().schema_version, 4);
    }
    db.close().await;
}
