use super::{backup_format as format, backups::RestoreMode, Database};
use crate::domain::{
    meal_library::{CookedFilter, MealSearch},
    meals::MealInput,
    planning::*,
    shopping::*,
};
use serde_json::{json, Value};
async fn text(db: &Database) -> String {
    let mut c = db.pool.acquire().await.unwrap();
    serde_json::to_string(&format::snapshot(&mut c).await.unwrap()).unwrap()
}
async fn fixture(db: &Database) {
    let field=db.create_field(serde_json::from_value(json!({"label":"Enjoyed","kind":"boolean","description":"","options":[],"minimum":null,"maximum":null,"unitLabel":null,"sortOrder":0})).unwrap()).await.unwrap();
    let mut meal: MealInput =
        serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap();
    meal.custom_fields = serde_json::from_value(
        json!([{"fieldId":field.id,"value":{"kind":"boolean","value":false}}]),
    )
    .unwrap();
    let meal = db.create_meal(meal).await.unwrap();
    for (id, servings) in [("one", "1 ½"), ("two", "4")] {
        let week = db.get_week("2020-12-28").await.unwrap();
        db.apply_plan(PlanMutation {
            action_id: format!("add-{id}"),
            week_start: week.week_start,
            expected_revision: week.revision,
            operation: PlanOperation::Add {
                entry_id: id.into(),
                meal_id: meal.id.clone(),
                local_date: Some("2020-12-29".into()),
                servings: servings.into(),
            },
        })
        .await
        .unwrap();
    }
    let week = db.get_week("2020-12-28").await.unwrap();
    db.apply_plan(PlanMutation {
        action_id: "cook-one".into(),
        week_start: week.week_start,
        expected_revision: week.revision,
        operation: PlanOperation::SetStatus {
            entry_id: "one".into(),
            status: PlanStatus::Cooked,
            cooked_date: Some("2020-12-29".into()),
            confirmed: false,
        },
    })
    .await
    .unwrap();
    let shopping = db
        .change_shopping(ShoppingMutation {
            action_id: "generate".into(),
            week_start: "2020-12-28".into(),
            expected_revision: 0,
            operation: ShoppingOperation::Refresh,
        })
        .await
        .unwrap();
    let id = shopping
        .items
        .iter()
        .find(|i| i.item.ingredient_id.as_deref() == Some("ingredient-broccoli"))
        .unwrap()
        .item
        .id
        .clone();
    let shopping = db
        .change_shopping(ShoppingMutation {
            action_id: "adjust".into(),
            week_start: "2020-12-28".into(),
            expected_revision: shopping.revision,
            operation: ShoppingOperation::Adjust {
                item_id: id.clone(),
                amount: "1 ½".into(),
                unit_id: Some("kg".into()),
            },
        })
        .await
        .unwrap();
    let shopping = db
        .change_shopping(ShoppingMutation {
            action_id: "check".into(),
            week_start: "2020-12-28".into(),
            expected_revision: shopping.revision,
            operation: ShoppingOperation::Check {
                item_id: id,
                checked: true,
            },
        })
        .await
        .unwrap();
    db.change_shopping(serde_json::from_value(json!({"actionId":"manual","weekStart":"2020-12-28","expectedRevision":shopping.revision,"operation":{"kind":"manual","itemId":"soap","label":"Soap","amount":"unknown","unitId":null,"sectionId":null}})).unwrap()).await.unwrap();
    let mut prefs = db.usability_preferences().await.unwrap();
    prefs.clipboard_url_prefill = true;
    prefs.section_order.reverse();
    db.save_usability_preferences(prefs).await.unwrap();
    sqlx::query("INSERT INTO import_record(id,meal_id,source_url,provenance_json) VALUES ('original',?,'https://example.com/owned','{\"originalYield\":\"4 servings\"}')").bind(&meal.id).execute(&db.pool).await.unwrap();
}
#[tokio::test]
async fn export_replace_roundtrip_preserves_household_data_excludes_browser_and_restarts() {
    let source = tempfile::tempdir().unwrap();
    let db = Database::open(source.path()).await.unwrap();
    fixture(&db).await;
    std::fs::create_dir(source.path().join("recipe-browser-profile")).unwrap();
    std::fs::write(
        source.path().join("recipe-browser-profile/Cookies"),
        "secret-cookie",
    )
    .unwrap();
    let exported = db.export_backup().await.unwrap();
    let contents = std::fs::read_to_string(&exported.path).unwrap();
    assert!(!contents.contains("secret-cookie"));
    assert!(!contents.contains("Cookies"));
    let expected = text(&db).await;
    let target = tempfile::tempdir().unwrap();
    let restored = Database::open(target.path()).await.unwrap();
    let preview = restored
        .preview_restore(&contents, RestoreMode::Replace)
        .await
        .unwrap();
    assert_eq!(preview.meals, 1);
    assert!(restored
        .restore_backup(
            &contents,
            RestoreMode::Replace,
            &preview.expected_state,
            false
        )
        .await
        .is_err());
    let recovery = restored
        .restore_backup(
            &contents,
            RestoreMode::Replace,
            &preview.expected_state,
            true,
        )
        .await
        .unwrap();
    assert!(std::path::Path::new(&recovery.path).is_file());
    assert_eq!(text(&restored).await, expected);
    restored.close().await;
    let restored = Database::open(target.path()).await.unwrap();
    assert_eq!(text(&restored).await, expected);
    assert_eq!(
        restored.get_week("2020-12-28").await.unwrap().entries.len(),
        2
    );
    let shopping = restored.get_shopping("2020-12-28").await.unwrap();
    assert!(shopping
        .items
        .iter()
        .any(|i| i.item.checked && i.item.adjustment.is_some()));
    assert!(shopping.items.iter().any(|i| i.item.label == "Soap"));
    assert!(
        restored
            .usability_preferences()
            .await
            .unwrap()
            .clipboard_url_prefill
    );
    // Same-data merge is a no-op, including durable action receipts.
    let review = restored
        .preview_restore(&contents, RestoreMode::Merge)
        .await
        .unwrap();
    restored
        .restore_backup(&contents, RestoreMode::Merge, &review.expected_state, true)
        .await
        .unwrap();
    assert_eq!(text(&restored).await, expected);
}
#[tokio::test]
async fn merge_adds_new_records_rejects_conflicts_and_detects_concurrent_changes() {
    let source = tempfile::tempdir().unwrap();
    let source_db = Database::open(source.path()).await.unwrap();
    source_db
        .create_meal(
            serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap(),
        )
        .await
        .unwrap();
    let incoming = text(&source_db).await;
    let target = tempfile::tempdir().unwrap();
    let db = Database::open(target.path()).await.unwrap();
    let review = db
        .preview_restore(&incoming, RestoreMode::Merge)
        .await
        .unwrap();
    db.restore_backup(&incoming, RestoreMode::Merge, &review.expected_state, true)
        .await
        .unwrap();
    let meal = db.list_meals().await.unwrap().remove(0);
    let mut edited = meal.data;
    edited.notes = "Household adjustment".into();
    db.update_meal(&meal.id, meal.revision, edited)
        .await
        .unwrap();
    let unchanged = text(&db).await;
    assert!(db
        .preview_restore(&incoming, RestoreMode::Merge)
        .await
        .is_err());
    assert_eq!(text(&db).await, unchanged);
    let preview = db
        .preview_restore(&incoming, RestoreMode::Replace)
        .await
        .unwrap();
    let mut prefs = db.usability_preferences().await.unwrap();
    prefs.clipboard_url_prefill = false;
    db.save_usability_preferences(prefs).await.unwrap();
    assert_eq!(
        db.restore_backup(
            &incoming,
            RestoreMode::Replace,
            &preview.expected_state,
            true
        )
        .await
        .unwrap_err()
        .code,
        "conflict"
    );
}
#[tokio::test]
async fn bad_backups_and_write_failures_never_erase_current_data() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    fixture(&db).await;
    let original = text(&db).await;
    for bad in ["{}".into(), "[".into(), "x".repeat(format::LIMIT + 1)] {
        assert!(db
            .preview_restore(&bad, RestoreMode::Replace)
            .await
            .is_err());
    }
    let base: Value = serde_json::from_str(&original).unwrap();
    for field in ["formatVersion", "schemaVersion"] {
        let mut bad = base.clone();
        bad[field] = json!(999);
        assert!(db
            .preview_restore(&bad.to_string(), RestoreMode::Replace)
            .await
            .is_err());
    }
    let mut old = base.clone();
    old["schemaVersion"] = json!(7);
    assert!(db
        .preview_restore(&old.to_string(), RestoreMode::Replace)
        .await
        .is_err());
    let mutate = |name: &str, column: &str, value: Value| {
        let mut bad = base.clone();
        let table = bad["tables"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|t| t["name"] == name)
            .unwrap();
        let index = table["columns"]
            .as_array()
            .unwrap()
            .iter()
            .position(|v| v == column)
            .unwrap();
        table["rows"][0][index] = value;
        bad.to_string()
    };
    for bad in [
        mutate("meal", "servings_denominator", json!(0)),
        mutate("meal", "servings_numerator", json!(u64::MAX)),
        mutate("meal", "servings_numerator", json!(1.5)),
        mutate("meal_ingredient", "ingredient_id", json!("missing")),
        mutate("ingredient", "normalized_name", json!("bogus")),
        mutate("unit", "factor_numerator", json!(1234)),
        mutate("custom_field_definition", "options_json", json!("broken")),
        mutate(
            "shopping_list",
            "document_json",
            json!("{\"version\":99,\"items\":[]}"),
        ),
    ] {
        assert!(db
            .preview_restore(&bad, RestoreMode::Replace)
            .await
            .is_err());
    }
    let table = base["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "shopping_list")
        .unwrap();
    let column = table["columns"]
        .as_array()
        .unwrap()
        .iter()
        .position(|c| c == "document_json")
        .unwrap();
    let document: Value = serde_json::from_str(table["rows"][0][column].as_str().unwrap()).unwrap();
    for (field, value) in [
        ("unitId", json!("missing-unit")),
        ("adjustment", json!({"amount":"-1","unitId":"kg"})),
    ] {
        let mut invalid = document.clone();
        invalid["items"][0][field] = value;
        let bad = mutate("shopping_list", "document_json", json!(invalid.to_string()));
        assert!(db
            .preview_restore(&bad, RestoreMode::Replace)
            .await
            .is_err());
    }
    assert_eq!(text(&db).await, original);
    let review = db
        .preview_restore(&original, RestoreMode::Replace)
        .await
        .unwrap();
    // Simulated storage fault after backup creation and before restore commit.
    sqlx::query("CREATE TRIGGER test_failure BEFORE INSERT ON ingredient BEGIN SELECT RAISE(ABORT,'simulated interruption'); END").execute(&db.pool).await.unwrap();
    assert!(db
        .restore_backup(
            &original,
            RestoreMode::Replace,
            &review.expected_state,
            true
        )
        .await
        .is_err());
    assert_eq!(text(&db).await, original);
    sqlx::query("DROP TRIGGER test_failure")
        .execute(&db.pool)
        .await
        .unwrap();
    // An unwritable backup destination stops restore before deletion.
    let blocked = tempfile::tempdir().unwrap();
    let other = Database::open(blocked.path()).await.unwrap();
    let before = text(&other).await;
    let review = other
        .preview_restore(&original, RestoreMode::Replace)
        .await
        .unwrap();
    std::fs::write(blocked.path().join("backups"), "not a directory").unwrap();
    assert!(other
        .restore_backup(
            &original,
            RestoreMode::Replace,
            &review.expected_state,
            true
        )
        .await
        .is_err());
    assert_eq!(text(&other).await, before);
}
#[tokio::test]
async fn personal_library_search_generation_and_snapshot_have_bounded_runtime() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let mut ids = vec![];
    for index in 0..200 {
        let mut meal: MealInput =
            serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap();
        meal.title = format!("Household recipe {index:03}");
        for line in &mut meal.ingredients {
            line.id = format!("{}-{index}", line.id);
        }
        ids.push(db.create_meal(meal).await.unwrap().id);
    }
    for (index, id) in ids.iter().take(21).enumerate() {
        let week = db.get_week("2020-12-28").await.unwrap();
        db.apply_plan(PlanMutation {
            action_id: format!("add-{index}"),
            week_start: week.week_start,
            expected_revision: week.revision,
            operation: PlanOperation::Add {
                entry_id: format!("entry-{index}"),
                meal_id: id.clone(),
                local_date: None,
                servings: "3 ½".into(),
            },
        })
        .await
        .unwrap();
    }
    let start = std::time::Instant::now();
    let result = db
        .search_meals(MealSearch {
            text: "Household recipe 1".into(),
            ingredient_id: None,
            tag: "".into(),
            field: None,
            cooked: CookedFilter::Any,
            cooked_from: None,
            cooked_to: None,
        })
        .await
        .unwrap();
    assert_eq!(result.meals.len(), 100);
    let shopping = db
        .change_shopping(ShoppingMutation {
            action_id: "performance".into(),
            week_start: "2020-12-28".into(),
            expected_revision: 0,
            operation: ShoppingOperation::Refresh,
        })
        .await
        .unwrap();
    assert!(shopping.items.iter().any(|i| i.item.sources.len() == 21));
    let exported = db.export_backup().await.unwrap();
    let payload = std::fs::read_to_string(exported.path).unwrap();
    let review = db
        .preview_restore(&payload, RestoreMode::Replace)
        .await
        .unwrap();
    assert_eq!(review.meals, 200);
    assert!(
        start.elapsed().as_secs() < 20,
        "200-meal search/generation/export/validation exceeded a generous 20-second smoke budget"
    );
}

#[tokio::test]
async fn backup_restores_preserved_legacy_temperature_lines_without_inventing_conversion() {
    let source = tempfile::tempdir().unwrap();
    let db = Database::open(source.path()).await.unwrap();
    fixture(&db).await;
    sqlx::query(
        "UPDATE meal_ingredient SET unit_id='celsius' WHERE ingredient_id='ingredient-broccoli'",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let current = db.get_shopping("2020-12-28").await.unwrap();
    db.change_shopping(ShoppingMutation {
        action_id: "legacy-refresh".into(),
        week_start: "2020-12-28".into(),
        expected_revision: current.revision,
        operation: ShoppingOperation::Refresh,
    })
    .await
    .unwrap();
    let expected = text(&db).await;
    let target = tempfile::tempdir().unwrap();
    let restored = Database::open(target.path()).await.unwrap();
    let preview = restored
        .preview_restore(&expected, RestoreMode::Replace)
        .await
        .unwrap();
    restored
        .restore_backup(
            &expected,
            RestoreMode::Replace,
            &preview.expected_state,
            true,
        )
        .await
        .unwrap();
    assert_eq!(text(&restored).await, expected);
    assert!(restored
        .get_shopping("2020-12-28")
        .await
        .unwrap()
        .items
        .iter()
        .any(|i| i.item.unit_id.as_deref() == Some("celsius")
            && i.item.warnings.iter().any(|w| w.contains("Temperature"))));
}
