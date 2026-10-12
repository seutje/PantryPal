use super::Database;
use crate::domain::{
    meals::{MealIngredient, MealInput},
    measurements::{Mode, RuleInput},
    planning::*,
    shopping::*,
    validation::Amount,
};
const WEEK: &str = "2020-12-28";
#[tokio::test]
async fn practical_displays_and_copy_keep_exact_totals_overrides_and_restart_data() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let mut input = fixture();
    input.base_servings = Amount {
        numerator: 1,
        denominator: 1,
    };
    input.ingredients = vec![line(
        "oil",
        Some("ingredient-oil"),
        Some(1),
        4,
        Some("cup-us"),
    )];
    let meal = db.create_meal(input).await.unwrap();
    add(&db, "oil-entry", &meal.id, "1").await;
    let view = change(&db, ShoppingOperation::Refresh).await;
    let oil = find(&view, "ingredient-oil");
    assert_eq!(oil.display, "60 mL");
    assert_eq!(oil.item.numerator.as_deref(), Some("473176473"));
    assert_eq!(oil.item.denominator.as_deref(), Some("8000000"));
    assert!(view.text.contains("Olive oil — 60 mL (rounded up)"));
    assert!(view.text.contains("¼ US cup"));
    assert!(!view.text.contains("alternatives"));
    assert!(!view.text.contains("rounded up as needed"));
    assert!(!view.text.contains("0.06 L"));
    let id = oil.item.id.clone();
    let view = change(
        &db,
        ShoppingOperation::Adjust {
            item_id: id.clone(),
            amount: "59.2".into(),
            unit_id: Some("ml".into()),
        },
    )
    .await;
    let oil = find(&view, "ingredient-oil");
    assert_eq!(oil.display, "60 mL");
    assert_eq!(oil.item.adjustment.as_ref().unwrap().amount, "59.2");
    assert_eq!(oil.item.numerator.as_deref(), Some("473176473"));
    let text = view.text.clone();
    drop(db);
    let db = Database::open(dir.path()).await.unwrap();
    assert_eq!(db.get_shopping(WEEK).await.unwrap().text, text);
    let refreshed = change(&db, ShoppingOperation::Refresh).await;
    assert_eq!(
        find(&refreshed, "ingredient-oil")
            .item
            .adjustment
            .as_ref()
            .unwrap()
            .amount,
        "59.2"
    );
    let unknown = change(
        &db,
        ShoppingOperation::Adjust {
            item_id: id,
            amount: "to taste".into(),
            unit_id: None,
        },
    )
    .await;
    assert_eq!(find(&unknown, "ingredient-oil").display, "to taste");
    assert!(find(&unknown, "ingredient-oil").alternatives.is_empty());
}
#[tokio::test]
async fn missing_units_and_legacy_temperatures_remain_separate_without_temperature_scaling() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let mut input = fixture();
    input.ingredients = vec![
        line(
            "legacy-temp",
            Some("ingredient-broccoli"),
            Some(100),
            1,
            Some("g"),
        ),
        line("missing", Some("ingredient-broccoli"), Some(2), 1, None),
    ];
    let meal = db.create_meal(input).await.unwrap();
    // Phase 1 allowed temperature assignments. Preserve that historical data;
    // current editors correctly reject new grocery temperature assignments.
    sqlx::query("UPDATE meal_ingredient SET unit_id='celsius' WHERE id='legacy-temp'")
        .execute(&db.pool)
        .await
        .unwrap();
    add(&db, "entry", &meal.id, "8").await;
    let view = change(&db, ShoppingOperation::Refresh).await;
    assert_eq!(view.items.len(), 2);
    let temperature = view
        .items
        .iter()
        .find(|i| i.item.unit_id.as_deref() == Some("celsius"))
        .unwrap();
    assert_eq!(temperature.item.numerator.as_deref(), Some("100"));
    assert!(temperature
        .item
        .warnings
        .iter()
        .any(|w| w.contains("Temperature")));
    let missing = view
        .items
        .iter()
        .find(|i| i.item.unit_id.is_none())
        .unwrap();
    assert_eq!(missing.item.numerator.as_deref(), Some("4"));
    assert!(missing
        .item
        .warnings
        .iter()
        .any(|w| w.contains("Missing unit")));
    db.close().await;
}
#[tokio::test]
async fn generation_overflow_rolls_back_without_erasing_manual_edits() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    change(
        &db,
        ShoppingOperation::Manual {
            item_id: "keep".into(),
            label: "Soap".into(),
            amount: "2".into(),
            unit_id: None,
            section_id: None,
        },
    )
    .await;
    db.save_conversion_rule(
        None,
        RuleInput {
            ingredient_id: "ingredient-broccoli".into(),
            from_unit_id: "piece".into(),
            from_amount: "1".into(),
            to_unit_id: "kg".into(),
            to_amount: "1000000000".into(),
            approximate: true,
            notes: "Overflow fixture, not a density assumption".into(),
        },
    )
    .await
    .unwrap();
    let mut meal = fixture();
    meal.base_servings = Amount {
        numerator: 1,
        denominator: 1000000000,
    };
    meal.ingredients = vec![line(
        "overflow",
        Some("ingredient-broccoli"),
        Some(1000000000),
        1,
        Some("piece"),
    )];
    let meal = db.create_meal(meal).await.unwrap();
    add(&db, "huge", &meal.id, "1000000000").await;
    let action = ShoppingMutation {
        action_id: "overflow".into(),
        week_start: WEEK.into(),
        expected_revision: 1,
        operation: ShoppingOperation::Refresh,
    };
    assert_eq!(
        db.change_shopping(action).await.unwrap_err().code,
        "validation"
    );
    let view = db.get_shopping(WEEK).await.unwrap();
    assert_eq!(view.revision, 1);
    assert!(!view.generated);
    assert_eq!(view.items.len(), 1);
    assert_eq!(view.items[0].item.id, "keep");
    assert_eq!(view.items[0].display, "2");
    db.close().await;
}
fn fixture() -> MealInput {
    serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap()
}
fn line(
    id: &str,
    ingredient: Option<&str>,
    n: Option<i64>,
    d: i64,
    unit: Option<&str>,
) -> MealIngredient {
    MealIngredient {
        id: id.into(),
        ingredient_id: ingredient.map(str::to_string),
        quantity: n.map(|n| Amount {
            numerator: n,
            denominator: d,
        }),
        amount_text: if n.is_none() {
            Some("to taste".into())
        } else {
            None
        },
        unit_id: unit.map(str::to_string),
        raw_text: format!("Original wording {id}"),
        source_convention: None,
        note: "Keep this note".into(),
        optional: false,
    }
}
async fn add(db: &Database, id: &str, meal: &str, servings: &str) {
    let week = db.get_week(WEEK).await.unwrap();
    db.apply_plan(PlanMutation {
        action_id: format!("plan-{id}"),
        week_start: WEEK.into(),
        expected_revision: week.revision,
        operation: PlanOperation::Add {
            entry_id: id.into(),
            meal_id: meal.into(),
            local_date: None,
            servings: servings.into(),
        },
    })
    .await
    .unwrap();
}
async fn change(db: &Database, operation: ShoppingOperation) -> ShoppingView {
    let old = db.get_shopping(WEEK).await.unwrap();
    db.change_shopping(ShoppingMutation {
        action_id: uuid::Uuid::new_v4().to_string(),
        week_start: WEEK.into(),
        expected_revision: old.revision,
        operation,
    })
    .await
    .unwrap()
}
fn find<'a>(view: &'a ShoppingView, id: &str) -> &'a DisplayItem {
    view.items
        .iter()
        .find(|i| i.item.ingredient_id.as_deref() == Some(id))
        .unwrap()
}
#[tokio::test]
async fn mixed_recipes_fractional_duplicates_regional_units_and_unknowns_count_exactly_once() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let mut a = fixture();
    a.ingredients = vec![
        line(
            "a-mass",
            Some("ingredient-broccoli"),
            Some(1),
            2,
            Some("kg"),
        ),
        line("a-cup", Some("ingredient-rice"), Some(1), 1, Some("cup-us")),
        line("a-salt", Some("ingredient-salt"), None, 1, Some("to-taste")),
        line("a-unresolved", None, Some(2), 1, Some("piece")),
    ];
    let a = db.create_meal(a).await.unwrap();
    let mut b = fixture();
    b.title = "Another supper".into();
    b.ingredients = vec![
        line(
            "b-mass",
            Some("ingredient-broccoli"),
            Some(250),
            1,
            Some("g"),
        ),
        line(
            "b-metric",
            Some("ingredient-rice"),
            Some(1),
            1,
            Some("cup-metric"),
        ),
        line(
            "b-imperial",
            Some("ingredient-rice"),
            Some(1),
            1,
            Some("cup-imperial"),
        ),
        line(
            "b-rice-mass",
            Some("ingredient-rice"),
            Some(20),
            1,
            Some("g"),
        ),
        line(
            "b-count",
            Some("ingredient-broccoli"),
            Some(1),
            1,
            Some("piece"),
        ),
        line(
            "b-pinch",
            Some("ingredient-salt"),
            Some(2),
            1,
            Some("pinch"),
        ),
    ];
    let b = db.create_meal(b).await.unwrap();
    add(&db, "first", &a.id, "2").await;
    add(&db, "duplicate", &a.id, "1 1/2").await;
    add(&db, "second", &b.id, "4").await;
    let week = db.get_week(WEEK).await.unwrap();
    db.apply_plan(PlanMutation {
        action_id: "cook-first".into(),
        week_start: WEEK.into(),
        expected_revision: week.revision,
        operation: PlanOperation::SetStatus {
            entry_id: "first".into(),
            status: PlanStatus::Cooked,
            cooked_date: Some("2020-12-29".into()),
            confirmed: false,
        },
    })
    .await
    .unwrap();
    let mut prefs = db.measurement_preferences().await.unwrap();
    prefs.mode = Mode::Metric;
    db.save_measurement_preferences(prefs).await.unwrap();
    let view = change(&db, ShoppingOperation::Refresh).await;
    let broccoli = view
        .items
        .iter()
        .find(|i| {
            i.item.ingredient_id.as_deref() == Some("ingredient-broccoli")
                && i.item.unit_id.as_deref() == Some("g")
        })
        .unwrap();
    assert_eq!(broccoli.item.numerator.as_deref(), Some("1375"));
    assert_eq!(broccoli.item.denominator.as_deref(), Some("2"));
    assert_eq!(broccoli.item.sources.len(), 3);
    let rice = view
        .items
        .iter()
        .find(|i| i.item.unit_id.as_deref() == Some("ml"))
        .unwrap();
    let expected = crate::domain::measurements::rational::Rational::new(2365882365, 10000000)
        .unwrap()
        .checked_mul(crate::domain::measurements::rational::Rational::new(7, 8).unwrap())
        .unwrap()
        .checked_add(
            crate::domain::measurements::rational::Rational::new(250000000 + 284130625, 1000000)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        rice.item.numerator.as_deref(),
        Some(expected.n.to_string().as_str())
    );
    assert_eq!(
        rice.item.denominator.as_deref(),
        Some(expected.d.to_string().as_str())
    );
    assert_eq!(
        view.items
            .iter()
            .map(|i| i.item.sources.len())
            .sum::<usize>(),
        14
    );
    assert_eq!(
        view.items
            .iter()
            .filter(|i| i.item.ingredient_id.is_none())
            .count(),
        2
    );
    assert_eq!(
        view.items
            .iter()
            .filter(|i| i.item.numerator.is_none())
            .count(),
        2
    );
    assert!(view
        .items
        .iter()
        .any(|i| i.item.warnings.iter().any(|w| w.contains("Informal"))));
    assert!(view.items.iter().any(|i| i
        .item
        .warnings
        .iter()
        .any(|w| w.contains("Separate quantity"))));
    assert_eq!(rice.item.sources[0].line.raw_text, "Original wording a-cup");
    assert_eq!(rice.item.sources[0].line.unit_id.as_deref(), Some("cup-us"));
    assert!(!view.stale);
    assert!(view
        .items
        .iter()
        .filter(|i| i.item.numerator.is_none())
        .all(|i| i.display == "to taste"));
    let week = db.get_week(WEEK).await.unwrap();
    db.apply_plan(PlanMutation {
        action_id: "skip-duplicate".into(),
        week_start: WEEK.into(),
        expected_revision: week.revision,
        operation: PlanOperation::SetStatus {
            entry_id: "duplicate".into(),
            status: PlanStatus::Skipped,
            cooked_date: None,
            confirmed: false,
        },
    })
    .await
    .unwrap();
    assert!(db.get_shopping(WEEK).await.unwrap().stale);
    let updated = change(&db, ShoppingOperation::Refresh).await;
    assert_eq!(
        updated
            .items
            .iter()
            .map(|i| i.item.sources.len())
            .sum::<usize>(),
        10
    );
    db.close().await;
}
#[tokio::test]
async fn refresh_preserves_overrides_manual_checks_exclusions_orphans_and_reappearing_keys() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let meal = db.create_meal(fixture()).await.unwrap();
    add(&db, "entry", &meal.id, "4").await;
    let initial = change(&db, ShoppingOperation::Refresh).await;
    let id = find(&initial, "ingredient-broccoli").item.id.clone();
    change(
        &db,
        ShoppingOperation::Adjust {
            item_id: id.clone(),
            amount: "1 1/2".into(),
            unit_id: Some("kg".into()),
        },
    )
    .await;
    change(
        &db,
        ShoppingOperation::Check {
            item_id: id.clone(),
            checked: true,
        },
    )
    .await;
    change(
        &db,
        ShoppingOperation::Exclude {
            item_id: id.clone(),
            excluded: true,
        },
    )
    .await;
    change(
        &db,
        ShoppingOperation::Manual {
            item_id: "manual-soap".into(),
            label: "Soap".into(),
            amount: "2".into(),
            unit_id: Some("piece".into()),
            section_id: None,
        },
    )
    .await;
    change(
        &db,
        ShoppingOperation::Check {
            item_id: "manual-soap".into(),
            checked: true,
        },
    )
    .await;
    let mut data = meal.data.clone();
    data.ingredients[1].quantity = Some(Amount {
        numerator: 750,
        denominator: 1,
    });
    db.update_meal(&meal.id, 1, data).await.unwrap();
    let stale = db.get_shopping(WEEK).await.unwrap();
    assert!(stale.stale);
    assert_eq!(
        find(&stale, "ingredient-broccoli")
            .item
            .numerator
            .as_deref(),
        Some("500")
    );
    let refreshed = change(&db, ShoppingOperation::Refresh).await;
    let row = find(&refreshed, "ingredient-broccoli");
    assert_eq!(row.item.numerator.as_deref(), Some("750"));
    assert_eq!(row.display, "1½ kg");
    assert!(row.item.checked && row.item.excluded);
    assert!(refreshed.text.contains("[x] Soap"));
    assert!(!refreshed.text.contains("Broccoli"));
    let mut data = db.meal(&meal.id).await.unwrap().data;
    let removed = data.ingredients.remove(1);
    db.update_meal(&meal.id, 2, data.clone()).await.unwrap();
    let orphaned = change(&db, ShoppingOperation::Refresh).await;
    assert!(find(&orphaned, "ingredient-broccoli").item.orphaned);
    assert_eq!(
        orphaned
            .items
            .iter()
            .filter(|i| i.item.id == "manual-soap")
            .count(),
        1
    );
    data.ingredients.push(removed);
    db.update_meal(&meal.id, 3, data).await.unwrap();
    let restored = change(&db, ShoppingOperation::Refresh).await;
    let row = find(&restored, "ingredient-broccoli");
    assert!(!row.item.orphaned);
    assert!(row.item.checked && row.item.excluded);
    assert_eq!(row.item.id, id);
    let mut prefs = db.measurement_preferences().await.unwrap();
    prefs.mode = Mode::Us;
    db.save_measurement_preferences(prefs).await.unwrap();
    let changed_units = change(&db, ShoppingOperation::Refresh).await;
    assert_eq!(find(&changed_units, "ingredient-broccoli").item.id, id);
    assert_eq!(find(&changed_units, "ingredient-broccoli").display, "1½ kg");
    let revision = changed_units.revision;
    assert!(db
        .change_shopping(ShoppingMutation {
            action_id: "invalid-reset".into(),
            week_start: WEEK.into(),
            expected_revision: revision,
            operation: ShoppingOperation::Reset {
                item_id: id.clone(),
                confirmed: false
            }
        })
        .await
        .is_err());
    assert_eq!(db.get_shopping(WEEK).await.unwrap().revision, revision);
    change(
        &db,
        ShoppingOperation::Reset {
            item_id: id,
            confirmed: true,
        },
    )
    .await;
    db.close().await;
}
#[tokio::test]
async fn ingredient_rules_display_preferences_no_chaining_and_ambiguity_are_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let mut broccoli = db.ingredient("ingredient-broccoli").await.unwrap().data;
    broccoli.preferred_unit_id = Some("kg".into());
    db.update_ingredient("ingredient-broccoli", broccoli)
        .await
        .unwrap();
    db.save_conversion_rule(
        None,
        RuleInput {
            ingredient_id: "ingredient-broccoli".into(),
            from_unit_id: "piece".into(),
            from_amount: "1".into(),
            to_unit_id: "g".into(),
            to_amount: "200".into(),
            approximate: true,
            notes: "Per medium head".into(),
        },
    )
    .await
    .unwrap();
    let mut input = fixture();
    input.ingredients = vec![
        line("mass", Some("ingredient-broccoli"), Some(100), 1, Some("g")),
        line(
            "pieces",
            Some("ingredient-broccoli"),
            Some(2),
            1,
            Some("piece"),
        ),
    ];
    let meal = db.create_meal(input).await.unwrap();
    add(&db, "entry", &meal.id, "4").await;
    let view = change(&db, ShoppingOperation::Refresh).await;
    assert_eq!(view.items.len(), 1);
    assert_eq!(view.items[0].display, "500 g");
    assert!(view.items[0].item.approximate);
    assert_eq!(view.items[0].item.rule_ids.len(), 1);
    let mut prefs = db.measurement_preferences().await.unwrap();
    prefs.aggregate_compatible = false;
    db.save_measurement_preferences(prefs).await.unwrap();
    assert_eq!(change(&db, ShoppingOperation::Refresh).await.items.len(), 2);
    db.save_conversion_rule(
        None,
        RuleInput {
            ingredient_id: "ingredient-broccoli".into(),
            from_unit_id: "kg".into(),
            from_amount: "1".into(),
            to_unit_id: "piece".into(),
            to_amount: "3".into(),
            approximate: true,
            notes: "Different size".into(),
        },
    )
    .await
    .unwrap();
    let mut prefs = db.measurement_preferences().await.unwrap();
    prefs.aggregate_compatible = true;
    db.save_measurement_preferences(prefs).await.unwrap();
    let ambiguous = change(&db, ShoppingOperation::Refresh).await;
    assert_eq!(ambiguous.items.len(), 2);
    assert!(ambiguous.items.iter().any(|i| i
        .item
        .warnings
        .iter()
        .any(|w| w.contains("Separate quantity"))));
    // A piece->volume rule plus volume->mass is NOT a piece->mass rule.
    for rule in db.conversion_rules("ingredient-broccoli").await.unwrap() {
        db.retire_conversion_rule(&rule.id, true).await.unwrap();
    }
    for (from, to, n) in [("piece", "ml", "100"), ("ml", "g", "2")] {
        db.save_conversion_rule(
            None,
            RuleInput {
                ingredient_id: "ingredient-broccoli".into(),
                from_unit_id: from.into(),
                from_amount: "1".into(),
                to_unit_id: to.into(),
                to_amount: n.into(),
                approximate: true,
                notes: "Fixture".into(),
            },
        )
        .await
        .unwrap();
    }
    let safe = change(&db, ShoppingOperation::Refresh).await;
    assert_eq!(safe.items.len(), 2);
    let piece = safe
        .items
        .iter()
        .find(|i| i.item.sources[0].line.id == "pieces")
        .unwrap();
    assert_ne!(piece.item.unit_id.as_deref(), Some("kg"));
    assert!(piece
        .item
        .warnings
        .iter()
        .any(|w| w.contains("direct conversion")));
    db.close().await;
}
#[tokio::test]
async fn mutations_are_atomic_retry_safe_and_legacy_rows_survive_upgrade_and_edit() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    sqlx::query("INSERT INTO shopping_list(id,week_start) VALUES ('legacy-list',?)")
        .bind(WEEK)
        .execute(&db.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO shopping_item(id,list_id,manual_label,origin,has_override,override_json,checked,excluded) VALUES ('legacy-item','legacy-list','Paper towels','manual',1,'{\"amount\":\"two packs\"}',1,1)").execute(&db.pool).await.unwrap();
    let old = db.get_shopping(WEEK).await.unwrap();
    assert_eq!(old.items.len(), 1);
    assert!(old.items[0].item.checked && old.items[0].item.excluded);
    assert!(old.items[0].item.legacy.is_some());
    let action = ShoppingMutation {
        action_id: "once".into(),
        week_start: WEEK.into(),
        expected_revision: 0,
        operation: ShoppingOperation::Manual {
            item_id: "soap".into(),
            label: "Soap".into(),
            amount: "unknown".into(),
            unit_id: None,
            section_id: None,
        },
    };
    let added = db.change_shopping(action.clone()).await.unwrap();
    assert_eq!(
        db.change_shopping(action.clone()).await.unwrap().revision,
        1
    );
    assert_eq!(added.items.len(), 2);
    let mut different = action.clone();
    different.operation = ShoppingOperation::Refresh;
    assert_eq!(
        db.change_shopping(different).await.unwrap_err().code,
        "conflict"
    );
    for (amount, unit) in [("-1", None), ("1/0", None), ("2", Some("celsius"))] {
        assert!(db
            .change_shopping(ShoppingMutation {
                action_id: uuid::Uuid::new_v4().to_string(),
                expected_revision: 1,
                week_start: WEEK.into(),
                operation: ShoppingOperation::Adjust {
                    item_id: "soap".into(),
                    amount: amount.into(),
                    unit_id: unit.map(str::to_string)
                }
            })
            .await
            .is_err());
    }
    assert_eq!(db.get_shopping(WEEK).await.unwrap().revision, 1);
    change(&db, ShoppingOperation::Refresh).await;
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_item WHERE id='legacy-item' AND checked=1 AND excluded=1",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
    db.close().await;
    let db = Database::open(dir.path()).await.unwrap();
    assert_eq!(db.get_shopping(WEEK).await.unwrap().items.len(), 2);
    assert_eq!(db.change_shopping(action).await.unwrap().revision, 2);
    db.close().await;
}
#[tokio::test]
async fn section_order_rounding_large_exact_totals_and_copy_text_are_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let mut prefs = db.measurement_preferences().await.unwrap();
    prefs.fractions = false;
    prefs.decimal_places = 2;
    db.save_measurement_preferences(prefs).await.unwrap();
    let mut input = fixture();
    input.base_servings = Amount {
        numerator: 1,
        denominator: 1,
    };
    input.ingredients = vec![
        line(
            "large",
            Some("ingredient-broccoli"),
            Some(1000000000),
            1,
            Some("kg"),
        ),
        line("third", Some("ingredient-onion"), Some(1), 3, Some("piece")),
    ];
    let meal = db.create_meal(input).await.unwrap();
    add(&db, "large-entry", &meal.id, "1000000000").await;
    let view = change(&db, ShoppingOperation::Refresh).await;
    assert_eq!(
        find(&view, "ingredient-broccoli").item.numerator.as_deref(),
        Some("1000000000000000000000")
    );
    assert!(find(&view, "ingredient-onion").rounded);
    change(
        &db,
        ShoppingOperation::Manual {
            item_id: "manual".into(),
            label: "Soap\nHousehold".into(),
            amount: "2".into(),
            unit_id: None,
            section_id: Some("section-other".into()),
        },
    )
    .await;
    db.update_vocabulary(
        crate::domain::catalog::VocabularyKind::Section,
        "section-other",
        "Household",
        0,
    )
    .await
    .unwrap();
    db.update_vocabulary(
        crate::domain::catalog::VocabularyKind::Section,
        "section-produce",
        "Produce",
        99,
    )
    .await
    .unwrap();
    let view = change(&db, ShoppingOperation::Refresh).await;
    assert_eq!(view.items[0].section, "Household");
    assert!(view.text.contains("Soap Household"));
    assert!(view.text.contains("(rounded up)"));
    assert_eq!(view.text, db.get_shopping(WEEK).await.unwrap().text);
    db.close().await;
}
