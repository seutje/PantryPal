use super::Database;
use crate::domain::planning::*;
use serde_json::json;
fn mutation(week: &WeekView, op: PlanOperation) -> PlanMutation {
    PlanMutation {
        action_id: uuid::Uuid::new_v4().to_string(),
        week_start: week.week_start.clone(),
        expected_revision: week.revision,
        operation: op,
    }
}
fn add(id: &str, meal: &str, date: Option<&str>, amount: &str) -> PlanOperation {
    PlanOperation::Add {
        entry_id: id.into(),
        meal_id: meal.into(),
        local_date: date.map(str::to_string),
        servings: amount.into(),
    }
}
fn status(id: &str, state: PlanStatus, date: Option<&str>, confirmed: bool) -> PlanOperation {
    PlanOperation::SetStatus {
        entry_id: id.into(),
        status: state,
        cooked_date: date.map(str::to_string),
        confirmed,
    }
}
#[test]
fn local_monday_weeks_and_serving_boundaries() {
    assert_eq!(
        week_dates("2027-01-01").unwrap(),
        [
            "2026-12-28",
            "2026-12-29",
            "2026-12-30",
            "2026-12-31",
            "2027-01-01",
            "2027-01-02",
            "2027-01-03"
        ]
    );
    assert_eq!(week_dates("2024-03-01").unwrap()[3], "2024-02-29");
    assert_eq!(week_dates("2026-03-08").unwrap()[0], "2026-03-02");
    assert_eq!(week_dates("2026-11-01").unwrap()[0], "2026-10-26");
    for bad in [
        "2026-02-30",
        "2026-1-01",
        "0000-01-01",
        "10000-01-01",
        "9999-12-31",
    ] {
        assert!(week_dates(bad).is_err());
    }
    for bad in ["0", "-1", "unknown", "1/0", "1000000001"] {
        assert!(servings(bad).is_err());
    }
    assert_eq!(servings("1 ½").unwrap().numerator, 3);
    assert_eq!(servings("1 ½").unwrap().denominator, 2);
}
#[tokio::test]
async fn occurrence_servings_scale_week_and_shopping_without_changing_recipe_after_restart() {
    use crate::domain::shopping::{ShoppingMutation, ShoppingOperation};
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let meal = db
        .create_meal(
            serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap(),
        )
        .await
        .unwrap();
    let original = serde_json::to_value(&meal).unwrap();
    let week = db.get_week("2027-01-01").await.unwrap();
    let week = db
        .apply_plan(mutation(
            &week,
            add("one", &meal.id, Some("2027-01-01"), "4"),
        ))
        .await
        .unwrap();
    let week = db
        .apply_plan(mutation(
            &week,
            add("two", &meal.id, Some("2027-01-01"), "4"),
        ))
        .await
        .unwrap();
    let shopping = db
        .change_shopping(ShoppingMutation {
            action_id: "refresh-before".into(),
            week_start: week.week_start.clone(),
            expected_revision: 0,
            operation: ShoppingOperation::Refresh,
        })
        .await
        .unwrap();
    let week = db
        .apply_plan(mutation(
            &week,
            PlanOperation::Edit {
                entry_id: "one".into(),
                local_date: Some("2027-01-01".into()),
                servings: "2".into(),
            },
        ))
        .await
        .unwrap();
    assert_eq!(
        week.entries[0].quantity_displays,
        [Some("⅙".into()), Some("250".into()), None]
    );
    assert_eq!(
        week.entries[1].quantity_displays,
        [Some("⅓".into()), Some("500".into()), None]
    );
    assert_eq!(
        serde_json::to_value(db.meal(&meal.id).await.unwrap()).unwrap(),
        original
    );
    assert!(db.get_shopping(&week.week_start).await.unwrap().stale);
    let shopping = db
        .change_shopping(ShoppingMutation {
            action_id: "refresh-after".into(),
            week_start: week.week_start.clone(),
            expected_revision: shopping.revision,
            operation: ShoppingOperation::Refresh,
        })
        .await
        .unwrap();
    let broccoli = shopping
        .items
        .iter()
        .find(|i| i.item.ingredient_id.as_deref() == Some("ingredient-broccoli"))
        .unwrap();
    assert_eq!(broccoli.item.numerator.as_deref(), Some("750"));
    assert_eq!(broccoli.item.denominator.as_deref(), Some("1"));
    assert_eq!(broccoli.item.sources.len(), 2);
    let week = db
        .apply_plan(mutation(
            &week,
            PlanOperation::Edit {
                entry_id: "one".into(),
                local_date: Some("2027-01-01".into()),
                servings: "1 ½".into(),
            },
        ))
        .await
        .unwrap();
    assert_eq!(
        week.entries[0].quantity_displays,
        [Some("⅛".into()), Some("187½".into()), None]
    );
    assert_eq!(week.entries[1].servings_text, "4");
    let mut prefs = db.measurement_preferences().await.unwrap();
    prefs.fractions = false;
    prefs.decimal_places = 2;
    db.save_measurement_preferences(prefs).await.unwrap();
    drop(db);
    let db = Database::open(dir.path()).await.unwrap();
    let week = db.get_week("2027-01-01").await.unwrap();
    assert_eq!(week.entries[0].servings_text, "1 ½");
    assert_eq!(
        week.entries[0].quantity_displays[0].as_deref(),
        Some("0.13 (rounded; exact 1/8)")
    );
    assert_eq!(
        week.entries[0].quantity_displays[1].as_deref(),
        Some("187.5")
    );
    assert_eq!(
        serde_json::to_value(db.meal(&meal.id).await.unwrap()).unwrap(),
        original
    );
}
#[tokio::test]
async fn duplicate_entries_exact_actions_reordering_and_atomic_rejection() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let meal = db
        .create_meal(
            serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap(),
        )
        .await
        .unwrap();
    let week = db.get_week("2027-01-01").await.unwrap();
    assert_eq!(week.revision, 0);
    let first = mutation(&week, add("one", &meal.id, None, "1 ½"));
    let mut week = db.apply_plan(first.clone()).await.unwrap();
    assert_eq!(week.entries.len(), 1);
    assert_eq!(db.apply_plan(first.clone()).await.unwrap().revision, 1);
    let mut changed = first.clone();
    changed.operation = add("one", &meal.id, None, "9");
    assert!(db.apply_plan(changed).await.is_err());
    for amount in ["0", "to taste", "-2"] {
        assert!(db
            .apply_plan(mutation(&week, add("bad", &meal.id, None, amount)))
            .await
            .is_err());
    }
    assert!(db
        .apply_plan(mutation(
            &week,
            add("bad", &meal.id, Some("2027-01-04"), "4")
        ))
        .await
        .is_err());
    assert_eq!(db.get_week("2027-01-01").await.unwrap().revision, 1);
    week = db
        .apply_plan(mutation(&week, add("two", &meal.id, None, "4")))
        .await
        .unwrap();
    assert_eq!(week.entries.len(), 2);
    assert!(db
        .meal_history(&meal.id)
        .await
        .unwrap()
        .last_cooked
        .is_none());
    let stale = mutation(&week, add("stale", &meal.id, None, "4"));
    week = db
        .apply_plan(mutation(
            &week,
            PlanOperation::Reorder {
                local_date: None,
                entry_ids: vec!["two".into(), "one".into()],
            },
        ))
        .await
        .unwrap();
    assert_eq!(week.entries[0].id, "two");
    assert!(db.apply_plan(stale).await.is_err());
    assert!(db
        .apply_plan(mutation(
            &week,
            PlanOperation::Reorder {
                local_date: None,
                entry_ids: vec!["two".into(), "two".into()]
            }
        ))
        .await
        .is_err());
    week = db
        .apply_plan(mutation(
            &week,
            PlanOperation::Edit {
                entry_id: "one".into(),
                local_date: Some("2027-01-01".into()),
                servings: "2/3".into(),
            },
        ))
        .await
        .unwrap();
    assert_eq!(week.entries[1].servings_text, "2/3");
    db.retire_meal(&meal.id, true).await.unwrap();
    assert!(db
        .apply_plan(mutation(&week, add("retired-new", &meal.id, None, "4")))
        .await
        .is_err());
    assert!(db
        .get_week("2027-01-01")
        .await
        .unwrap()
        .library
        .meals
        .iter()
        .any(|m| m.meal.id == meal.id && m.meal.retired));
    week = db
        .apply_plan(mutation(
            &week,
            status("two", PlanStatus::Skipped, None, false),
        ))
        .await
        .unwrap();
    assert!(db
        .meal_history(&meal.id)
        .await
        .unwrap()
        .last_cooked
        .is_none());
    let remove = mutation(
        &week,
        PlanOperation::Remove {
            entry_id: "two".into(),
            confirmed: true,
        },
    );
    week = db.apply_plan(remove.clone()).await.unwrap();
    assert_eq!(week.entries.len(), 1);
    assert_eq!(db.apply_plan(first).await.unwrap().entries.len(), 1);
    assert_eq!(db.apply_plan(remove).await.unwrap().revision, week.revision);
    db.close().await;
}
#[tokio::test]
async fn cooking_undo_correction_and_removal_preserve_one_event_and_audit() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let meal = db
        .create_meal(
            serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap(),
        )
        .await
        .unwrap();
    let mut week = db.get_week("2020-12-28").await.unwrap();
    week = db
        .apply_plan(mutation(&week, add("cook-one", &meal.id, None, "4")))
        .await
        .unwrap();
    let cook = mutation(
        &week,
        status("cook-one", PlanStatus::Cooked, Some("2020-12-29"), false),
    );
    week = db.apply_plan(cook.clone()).await.unwrap();
    let history = db.meal_history(&meal.id).await.unwrap();
    assert_eq!(history.events.len(), 1);
    let id = history.events[0].id.clone();
    assert_eq!(history.last_cooked.as_deref(), Some("2020-12-29"));
    assert_eq!(
        db.apply_plan(cook.clone()).await.unwrap().revision,
        week.revision
    );
    assert!(db
        .apply_plan(mutation(
            &week,
            status("cook-one", PlanStatus::Planned, None, false)
        ))
        .await
        .is_err());
    assert!(db
        .apply_plan(mutation(
            &week,
            status("cook-one", PlanStatus::Cooked, Some("2099-01-01"), true)
        ))
        .await
        .is_err());
    week = db
        .apply_plan(mutation(
            &week,
            status("cook-one", PlanStatus::Planned, None, true),
        ))
        .await
        .unwrap();
    assert!(db
        .meal_history(&meal.id)
        .await
        .unwrap()
        .last_cooked
        .is_none());
    assert!(db.meal_history(&meal.id).await.unwrap().events[0].voided);
    db.apply_plan(cook).await.unwrap();
    assert!(db
        .meal_history(&meal.id)
        .await
        .unwrap()
        .last_cooked
        .is_none());
    week = db
        .apply_plan(mutation(
            &week,
            status("cook-one", PlanStatus::Cooked, Some("2020-12-30"), false),
        ))
        .await
        .unwrap();
    assert_eq!(db.meal_history(&meal.id).await.unwrap().events[0].id, id);
    assert_eq!(db.meal_history(&meal.id).await.unwrap().events.len(), 1);
    let event = db.meal_history(&meal.id).await.unwrap().events.remove(0);
    let correction = HistoryMutation {
        action_id: "correct-date".into(),
        meal_id: meal.id.clone(),
        operation: HistoryOperation::Correct {
            event_id: id.clone(),
            expected_revision: event.revision,
            local_date: "2020-12-31".into(),
            confirmed: true,
        },
    };
    db.change_history(correction.clone()).await.unwrap();
    db.change_history(correction).await.unwrap();
    let current = db.get_week(&week.week_start).await.unwrap();
    assert_eq!(current.revision, week.revision + 1);
    assert_eq!(current.entries[0].status, PlanStatus::Cooked);
    assert!(db
        .apply_plan(mutation(
            &week,
            PlanOperation::Remove {
                entry_id: "cook-one".into(),
                confirmed: true
            }
        ))
        .await
        .is_err());
    week = current;
    week = db
        .apply_plan(mutation(
            &week,
            PlanOperation::Edit {
                entry_id: "cook-one".into(),
                local_date: Some("2021-01-03".into()),
                servings: "5".into(),
            },
        ))
        .await
        .unwrap();
    assert_eq!(
        db.meal_history(&meal.id)
            .await
            .unwrap()
            .last_cooked
            .as_deref(),
        Some("2020-12-31")
    );
    week = db
        .apply_plan(mutation(
            &week,
            PlanOperation::Remove {
                entry_id: "cook-one".into(),
                confirmed: true,
            },
        ))
        .await
        .unwrap();
    assert!(week.entries.is_empty());
    let event = db.meal_history(&meal.id).await.unwrap().events.remove(0);
    assert!(event.plan_entry_id.is_none());
    assert_eq!(event.origin_plan_entry_id.as_deref(), Some("cook-one"));
    assert!(!event.voided);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM cooked_event_change")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 3);
    db.close().await;
}
#[tokio::test]
async fn backfill_undo_revisions_and_restart_keep_latest_actual_date() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let meal = db
        .create_meal(
            serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap(),
        )
        .await
        .unwrap();
    let record = HistoryMutation {
        action_id: "backfill".into(),
        meal_id: meal.id.clone(),
        operation: HistoryOperation::Record {
            event_id: "past-event".into(),
            local_date: "2020-02-29".into(),
        },
    };
    db.change_history(record.clone()).await.unwrap();
    db.change_history(record.clone()).await.unwrap();
    assert!(db
        .change_history(HistoryMutation {
            action_id: "invalid-future".into(),
            meal_id: meal.id.clone(),
            operation: HistoryOperation::Record {
                event_id: "bad".into(),
                local_date: "2099-01-01".into()
            }
        })
        .await
        .is_err());
    let undo = HistoryMutation {
        action_id: "undo".into(),
        meal_id: meal.id.clone(),
        operation: HistoryOperation::Undo {
            event_id: "past-event".into(),
            expected_revision: 1,
            confirmed: true,
        },
    };
    db.change_history(undo.clone()).await.unwrap();
    db.change_history(record).await.unwrap();
    assert!(db
        .meal_history(&meal.id)
        .await
        .unwrap()
        .last_cooked
        .is_none());
    db.close().await;
    let db = Database::open(dir.path()).await.unwrap();
    assert!(db.meal_history(&meal.id).await.unwrap().events[0].voided);
    db.change_history(undo).await.unwrap();
    let library=db.search_meals(serde_json::from_value(json!({"text":"","ingredientId":null,"tag":"","field":null,"cooked":"never","cookedFrom":null,"cookedTo":null})).unwrap()).await.unwrap();
    assert_eq!(library.meals.len(), 1);
    db.close().await;
}

#[tokio::test]
async fn schema_six_upgrade_preserves_plans_events_and_shopping_adjustments() {
    use sqlx::{
        migrate::Migrator,
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    };
    let directory = tempfile::tempdir().unwrap();
    let scripts = tempfile::tempdir().unwrap();
    for (name, sql) in [
        (
            "0001_catalog.sql",
            include_str!("../../migrations/0001_catalog.sql"),
        ),
        (
            "0002_meals.sql",
            include_str!("../../migrations/0002_meals.sql"),
        ),
        (
            "0003_planning_storage.sql",
            include_str!("../../migrations/0003_planning_storage.sql"),
        ),
        (
            "0004_seed_catalog.sql",
            include_str!("../../migrations/0004_seed_catalog.sql"),
        ),
        (
            "0005_measurement_preferences.sql",
            include_str!("../../migrations/0005_measurement_preferences.sql"),
        ),
        (
            "0006_recipe_amount_text.sql",
            include_str!("../../migrations/0006_recipe_amount_text.sql"),
        ),
    ] {
        std::fs::write(scripts.path().join(name), sql).unwrap();
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(directory.path().join("pantrypal.sqlite3"))
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await
        .unwrap();
    Migrator::new(scripts.path())
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();
    let old = Database { pool };
    let meal = old
        .create_meal(
            serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap(),
        )
        .await
        .unwrap();
    sqlx::query("INSERT INTO plan_entry(id,week_start,meal_id,servings_numerator,servings_denominator,status) VALUES ('legacy-plan','2020-12-28',?,8,2,'cooked')").bind(&meal.id).execute(&old.pool).await.unwrap();
    sqlx::query("INSERT INTO meal_cooked_event(id,meal_id,local_date,plan_entry_id) VALUES ('legacy-history',?,'2020-12-31','legacy-plan')").bind(&meal.id).execute(&old.pool).await.unwrap();
    sqlx::query("INSERT INTO shopping_list(id,week_start,generation_revision) VALUES ('legacy-shop','2020-12-28',2)").execute(&old.pool).await.unwrap();
    sqlx::query("INSERT INTO shopping_item(id,list_id,origin,manual_label,checked,has_override,override_json) VALUES ('manual','legacy-shop','manual','Soap',1,1,'{\"amount\":\"2 packs\"}')").execute(&old.pool).await.unwrap();
    old.close().await;
    let db = Database::open(directory.path()).await.unwrap();
    let week = db.get_week("2021-01-01").await.unwrap();
    assert_eq!(week.revision, 1);
    assert_eq!(week.entries[0].servings_text, "8/2");
    assert_eq!(week.entries[0].servings.numerator, 8);
    assert_eq!(
        week.entries[0]
            .cooked_event
            .as_ref()
            .unwrap()
            .origin_plan_entry_id
            .as_deref(),
        Some("legacy-plan")
    );
    assert_eq!(
        db.meal_history(&meal.id)
            .await
            .unwrap()
            .last_cooked
            .as_deref(),
        Some("2020-12-31")
    );
    assert_eq!(db.meal(&meal.id).await.unwrap(), meal);
    let edited = db
        .apply_plan(mutation(
            &week,
            PlanOperation::Edit {
                entry_id: "legacy-plan".into(),
                local_date: Some("2021-01-01".into()),
                servings: "8/2".into(),
            },
        ))
        .await
        .unwrap();
    assert_eq!(edited.entries[0].servings.numerator, 8);
    assert_eq!(edited.entries[0].servings.denominator, 2);
    let row: (bool, String) =
        sqlx::query_as("SELECT checked,override_json FROM shopping_item WHERE id='manual'")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert!(row.0);
    assert_eq!(row.1, "{\"amount\":\"2 packs\"}");
    assert_eq!(db.status().await.unwrap().schema_version, 8);
    db.close().await;
}
