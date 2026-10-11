use super::*;
use crate::domain::{
    catalog::{IngredientInput, VocabularyKind},
    fields::*,
    meals::{MealFieldValue, MealInput},
    validation::Amount,
};
use sqlx::{sqlite::SqliteConnectOptions, Row};
use tempfile::TempDir;

fn ingredient(name: &str) -> IngredientInput {
    IngredientInput {
        name: name.into(),
        category_id: Some("category-vegetable".into()),
        section_id: Some("section-produce".into()),
        preferred_unit_id: Some("g".into()),
        aliases: vec![format!("{name} leaves")],
        season_months: vec![5, 6],
        notes: "Keep cool.".into(),
    }
}
fn demo() -> MealInput {
    serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap()
}
fn field(kind: FieldKind) -> FieldInput {
    FieldInput {
        label: format!("Test {kind:?}"),
        kind,
        description: String::new(),
        options: vec![],
        minimum: None,
        maximum: None,
        unit_label: None,
        sort_order: 0,
    }
}

#[tokio::test]
async fn fresh_database_migrates_once_and_preserves_seed_edits() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    assert_eq!(db.status().await.unwrap().schema_version, 7);
    assert_eq!(db.status().await.unwrap().ingredient_count, 16);
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(foreign_keys, 1);
    assert_eq!(journal, "wal");
    let mut onion = db.ingredient("ingredient-onion").await.unwrap().data;
    onion.name = "Yellow onion".into();
    onion.notes = "Edited by household".into();
    db.update_ingredient("ingredient-onion", onion)
        .await
        .unwrap();
    db.retire_ingredient("ingredient-garlic", true)
        .await
        .unwrap();
    db.close().await;
    let reopened = Database::open(directory.path()).await.unwrap();
    assert_eq!(reopened.status().await.unwrap().ingredient_count, 15);
    assert_eq!(
        reopened
            .ingredient("ingredient-onion")
            .await
            .unwrap()
            .data
            .notes,
        "Edited by household"
    );
    assert!(
        reopened
            .ingredient("ingredient-garlic")
            .await
            .unwrap()
            .retired
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
        .fetch_one(&reopened.pool)
        .await
        .unwrap();
    assert_eq!(count, 7);
    reopened.close().await;
}

#[tokio::test]
async fn migration_from_existing_catalog_preserves_household_rows() {
    let directory = TempDir::new().unwrap();
    let scripts = TempDir::new().unwrap();
    std::fs::write(
        scripts.path().join("0001_catalog.sql"),
        include_str!("../../migrations/0001_catalog.sql"),
    )
    .unwrap();
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
    sqlx::query("INSERT INTO ingredient(id,name,normalized_name,notes) VALUES (?,?,?,?)")
        .bind("household-herb")
        .bind("Household herb")
        .bind("household herb")
        .bind("Keep my data")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
    let upgraded = Database::open(directory.path()).await.unwrap();
    assert_eq!(upgraded.status().await.unwrap().schema_version, 7);
    assert_eq!(upgraded.status().await.unwrap().ingredient_count, 17);
    assert_eq!(
        upgraded
            .ingredient("household-herb")
            .await
            .unwrap()
            .data
            .notes,
        "Keep my data"
    );
    upgraded.close().await;
}

#[tokio::test]
async fn future_and_changed_migrations_are_rejected_without_resetting_data() {
    for changed_checksum in [false, true] {
        let directory = TempDir::new().unwrap();
        let db = Database::open(directory.path()).await.unwrap();
        let created = db.create_ingredient(ingredient("Parsley")).await.unwrap();
        if changed_checksum {
            sqlx::query("UPDATE _sqlx_migrations SET checksum=x'01' WHERE version=1")
                .execute(&db.pool)
                .await
                .unwrap();
        } else {
            sqlx::query("INSERT INTO _sqlx_migrations(version,description,success,checksum,execution_time) VALUES (999,'future',1,x'01',0)").execute(&db.pool).await.unwrap();
        }
        db.close().await;
        let error = match Database::open(directory.path()).await {
            Ok(_) => panic!("incompatible data accepted"),
            Err(error) => error,
        };
        assert_eq!(error.code, "migration_failed");
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("pantrypal.sqlite3")),
        )
        .await
        .unwrap();
        let name: String = sqlx::query_scalar("SELECT name FROM ingredient WHERE id=?")
            .bind(created.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(name, "Parsley");
        pool.close().await;
    }
}

#[tokio::test]
async fn corrupt_database_is_not_recreated() {
    let directory = TempDir::new().unwrap();
    let file = directory.path().join("pantrypal.sqlite3");
    std::fs::write(&file, b"not a database").unwrap();
    assert!(Database::open(directory.path()).await.is_err());
    assert_eq!(std::fs::read(file).unwrap(), b"not a database");
}

#[tokio::test]
async fn catalog_crud_is_atomic_and_name_alias_namespace_is_unique() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let created = db.create_ingredient(ingredient("Parsley")).await.unwrap();
    let mut update = created.data.clone();
    update.name = "Flat leaf parsley".into();
    update.aliases = vec!["Italian parsley".into()];
    let updated = db.update_ingredient(&created.id, update).await.unwrap();
    assert_eq!(updated.data.name, "Flat leaf parsley");
    let mut collision = updated.data.clone();
    collision.name = "Changed title".into();
    collision.aliases = vec!["Onions".into()];
    assert_eq!(
        db.update_ingredient(&created.id, collision)
            .await
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(db.ingredient(&created.id).await.unwrap(), updated);
    assert!(db
        .create_ingredient(ingredient("  ITALIAN   parsley "))
        .await
        .is_err());
    let mut alias_to_name = ingredient("Basil");
    alias_to_name.aliases = vec![" Milk ".into()];
    assert!(db.create_ingredient(alias_to_name).await.is_err());
    assert!(db.retire_ingredient(&created.id, false).await.is_err());
    db.retire_ingredient(&created.id, true).await.unwrap();
    assert!(db.ingredient(&created.id).await.unwrap().retired);
    assert!(!db
        .catalog()
        .await
        .unwrap()
        .ingredients
        .iter()
        .any(|value| value.id == created.id));
    db.close().await;
}

#[tokio::test]
async fn vocabulary_crud_and_references_are_durable() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    for kind in [VocabularyKind::Category, VocabularyKind::Section] {
        let value = db.create_vocabulary(kind, "Herbs", 3).await.unwrap();
        let updated = db
            .update_vocabulary(kind, &value.id, "Fresh herbs", 4)
            .await
            .unwrap();
        assert_eq!(updated.name, "Fresh herbs");
        db.retire_vocabulary(kind, &value.id, true).await.unwrap();
        assert!(db.vocabulary(kind, &value.id).await.unwrap().retired);
    }
    let mut invalid = ingredient("Parsley");
    invalid.category_id = Some("missing".into());
    assert_eq!(
        db.create_ingredient(invalid).await.unwrap_err().code,
        "conflict"
    );
    db.close().await;
}

#[tokio::test]
async fn meal_crud_retains_exact_source_values_and_rolls_back_failed_children() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let created = db.create_meal(demo()).await.unwrap();
    assert_eq!(created.data, demo());
    assert!(created.data.ingredients[2].quantity.is_none());
    assert_eq!(
        created.data.ingredients[0].quantity,
        Some(Amount {
            numerator: 1,
            denominator: 3
        })
    );
    let mut changed = created.data.clone();
    changed.title = "Updated supper".into();
    changed.ingredients[0].note = "rinse".into();
    let updated = db
        .update_meal(&created.id, created.revision, changed)
        .await
        .unwrap();
    assert_eq!(updated.revision, 2);
    assert_eq!(
        db.update_meal(&created.id, 1, demo())
            .await
            .unwrap_err()
            .code,
        "conflict"
    );
    let mut bad = updated.data.clone();
    bad.title = "Should roll back".into();
    bad.ingredients[1].ingredient_id = Some("missing".into());
    assert!(db.update_meal(&created.id, 2, bad).await.is_err());
    assert_eq!(db.meal(&created.id).await.unwrap(), updated);
    db.retire_ingredient("ingredient-rice", true).await.unwrap();
    assert_eq!(
        db.meal(&created.id).await.unwrap().data.ingredients[0].ingredient_id,
        Some("ingredient-rice".into())
    );
    assert!(db.create_meal(demo()).await.is_err());
    let mut edited = updated.data.clone();
    edited.notes = "Referenced retired rice remains editable".into();
    let edited = db.update_meal(&created.id, 2, edited).await.unwrap();
    assert_eq!(edited.revision, 3);
    assert!(db.retire_meal(&created.id, false).await.is_err());
    db.retire_meal(&created.id, true).await.unwrap();
    assert!(db.meal(&created.id).await.unwrap().retired);
    assert!(db.list_meals().await.unwrap().is_empty());
    db.close().await;
}

#[tokio::test]
async fn custom_fields_validate_types_and_preserve_unknown_false_and_retired_values() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let definition = db.create_field(field(FieldKind::Boolean)).await.unwrap();
    let mut input = demo();
    input.custom_fields.push(MealFieldValue {
        field_id: definition.id.clone(),
        value: None,
    });
    let unknown = db.create_meal(input).await.unwrap();
    assert_eq!(
        db.meal(&unknown.id).await.unwrap().data.custom_fields[0].value,
        None
    );
    let mut input = unknown.data.clone();
    input.custom_fields[0].value = Some(FieldValue::Boolean(false));
    let false_meal = db.update_meal(&unknown.id, 1, input).await.unwrap();
    assert_eq!(
        false_meal.data.custom_fields[0].value,
        Some(FieldValue::Boolean(false))
    );
    let mut bad = false_meal.data.clone();
    bad.custom_fields[0].value = Some(FieldValue::Text("false".into()));
    assert!(db.update_meal(&unknown.id, 2, bad).await.is_err());
    let mut renamed = definition.data.clone();
    renamed.label = "Kids enjoyed?".into();
    db.update_field(&definition.id, renamed.clone())
        .await
        .unwrap();
    let mut type_change = renamed;
    type_change.kind = FieldKind::Text;
    assert!(db.update_field(&definition.id, type_change).await.is_err());
    assert!(db.retire_field(&definition.id, false).await.is_err());
    db.retire_field(&definition.id, true).await.unwrap();
    let mut omitted = false_meal.data.clone();
    omitted.custom_fields.clear();
    omitted.notes = "Unrelated edit".into();
    let preserved = db.update_meal(&unknown.id, 2, omitted).await.unwrap();
    assert_eq!(
        preserved.data.custom_fields[0].value,
        Some(FieldValue::Boolean(false))
    );
    assert!(db.list_fields().await.unwrap().is_empty());
    db.close().await;
}

#[test]
fn field_validation_covers_all_kinds_and_bounds() {
    for kind in [
        FieldKind::Boolean,
        FieldKind::Select,
        FieldKind::Number,
        FieldKind::Date,
        FieldKind::Text,
        FieldKind::Rating,
    ] {
        let mut definition = field(kind.clone());
        if kind == FieldKind::Select {
            definition.options = vec![FieldOption {
                id: "easy".into(),
                label: "Easy".into(),
            }];
        }
        if kind == FieldKind::Rating {
            definition.minimum = Some(1.0);
            definition.maximum = Some(5.0);
        }
        definition.validate().unwrap();
        definition.validate_value(&None).unwrap();
        let (valid, invalid) = match kind {
            FieldKind::Boolean => (FieldValue::Boolean(false), FieldValue::Text("false".into())),
            FieldKind::Select => (
                FieldValue::Select("easy".into()),
                FieldValue::Select("unknown".into()),
            ),
            FieldKind::Number => (FieldValue::Number(1.5), FieldValue::Number(f64::NAN)),
            FieldKind::Date => (
                FieldValue::Date("2024-02-29".into()),
                FieldValue::Date("2025-02-29".into()),
            ),
            FieldKind::Text => (
                FieldValue::Text("line one\nline two".into()),
                FieldValue::Boolean(true),
            ),
            FieldKind::Rating => (FieldValue::Rating(5.0), FieldValue::Rating(2.5)),
        };
        definition.validate_value(&Some(valid)).unwrap();
        assert!(definition.validate_value(&Some(invalid)).is_err());
    }
    let mut number = field(FieldKind::Number);
    number.minimum = Some(10.0);
    number.maximum = Some(1.0);
    assert!(number.validate().is_err());
    assert!(Amount {
        numerator: 1,
        denominator: 0
    }
    .validate(true)
    .is_err());
    assert!(Amount {
        numerator: 0,
        denominator: 1
    }
    .validate(true)
    .is_err());
    assert!(Amount {
        numerator: 0,
        denominator: 1
    }
    .validate(false)
    .is_ok());
    let mut invalid = ingredient("Parsley");
    invalid.season_months = vec![13];
    assert!(invalid.validate().is_err());
}

#[tokio::test]
async fn foreign_keys_and_quantity_constraints_reject_bad_storage() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let meal = db.create_meal(demo()).await.unwrap();
    assert!(
        sqlx::query("DELETE FROM ingredient WHERE id='ingredient-rice'")
            .execute(&db.pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("UPDATE meal_ingredient SET quantity_denominator=0 WHERE meal_id=?")
            .bind(&meal.id)
            .execute(&db.pool)
            .await
            .is_err()
    );
    assert!(sqlx::query(
        "UPDATE meal_ingredient SET quantity_numerator=NULL WHERE id='fixture-rice-line'"
    )
    .execute(&db.pool)
    .await
    .is_err());
    assert!(
        sqlx::query("UPDATE ingredient SET category_id='missing' WHERE id='ingredient-rice'")
            .execute(&db.pool)
            .await
            .is_err()
    );
    let violations = sqlx::query("PRAGMA foreign_key_check")
        .fetch_all(&db.pool)
        .await
        .unwrap();
    assert!(violations.is_empty());
    db.close().await;
}

#[tokio::test]
async fn planning_and_shopping_storage_preserves_duplicates_and_manual_adjustments() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let meal = db.create_meal(demo()).await.unwrap();
    for id in ["first", "second"] {
        sqlx::query("INSERT INTO plan_entry(id,week_start,meal_id,servings_numerator,servings_denominator) VALUES (?,'2026-10-12',?,2,1)").bind(id).bind(&meal.id).execute(&db.pool).await.unwrap();
    }
    let history: i64 = sqlx::query_scalar("SELECT count(*) FROM meal_cooked_event")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(history, 0);
    sqlx::query("INSERT INTO meal_cooked_event(id,meal_id,local_date,plan_entry_id) VALUES ('cooked',?,'2026-10-12','first')").bind(&meal.id).execute(&db.pool).await.unwrap();
    assert!(sqlx::query("INSERT INTO meal_cooked_event(id,meal_id,local_date,plan_entry_id) VALUES ('again',?,'2026-10-12','first')").bind(&meal.id).execute(&db.pool).await.is_err());
    sqlx::query("INSERT INTO shopping_list(id,week_start) VALUES ('list','2026-10-12')")
        .execute(&db.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO shopping_item(id,list_id,manual_label,origin,has_override,override_json,checked,excluded) VALUES ('manual','list','Dish soap','manual',1,'{\"quantity\":null}',1,1)").execute(&db.pool).await.unwrap();
    db.retire_meal(&meal.id, true).await.unwrap();
    db.close().await;
    let db = Database::open(directory.path()).await.unwrap();
    let entries: i64 = sqlx::query_scalar("SELECT count(*) FROM plan_entry")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(entries, 2);
    let row = sqlx::query("SELECT * FROM shopping_item WHERE id='manual'")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert!(row.get::<bool, _>("has_override"));
    assert!(row.get::<bool, _>("checked"));
    assert!(row.get::<bool, _>("excluded"));
    assert_eq!(row.get::<String, _>("override_json"), "{\"quantity\":null}");
    db.close().await;
}

#[tokio::test]
async fn explicit_transaction_rollback_preserves_existing_rows() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let mut tx = db.pool.begin().await.unwrap();
    sqlx::query("UPDATE ingredient SET notes='interrupted write' WHERE id='ingredient-rice'")
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(
        db.ingredient("ingredient-rice").await.unwrap().data.notes,
        ""
    );
    db.close().await;
}

#[tokio::test]
async fn failed_migration_rolls_back_its_schema_and_data() {
    let directory = TempDir::new().unwrap();
    let scripts = TempDir::new().unwrap();
    std::fs::write(
        scripts.path().join("0001_catalog.sql"),
        include_str!("../../migrations/0001_catalog.sql"),
    )
    .unwrap();
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(directory.path().join("test.sqlite3"))
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
    sqlx::query("INSERT INTO ingredient(id,name,normalized_name) VALUES ('keep','Keep','keep')")
        .execute(&pool)
        .await
        .unwrap();
    std::fs::write(scripts.path().join("0002_invalid.sql"),"CREATE TABLE partial (id INTEGER); UPDATE ingredient SET notes='should roll back'; INSERT INTO missing_table VALUES (1);").unwrap();
    assert!(Migrator::new(scripts.path())
        .await
        .unwrap()
        .run(&pool)
        .await
        .is_err());
    let notes: String = sqlx::query_scalar("SELECT notes FROM ingredient WHERE id='keep'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let tables: i64 = sqlx::query_scalar("SELECT count(*) FROM sqlite_master WHERE name='partial'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(notes, "");
    assert_eq!(tables, 0);
    pool.close().await;
}

#[tokio::test]
async fn retired_vocabulary_keeps_existing_ingredients_editable() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let created = db.create_ingredient(ingredient("Parsley")).await.unwrap();
    db.retire_vocabulary(VocabularyKind::Category, "category-vegetable", true)
        .await
        .unwrap();
    db.retire_vocabulary(VocabularyKind::Section, "section-produce", true)
        .await
        .unwrap();
    let mut input = created.data;
    input.notes = "Still editable".into();
    db.update_ingredient(&created.id, input).await.unwrap();
    assert!(db.create_ingredient(ingredient("Basil")).await.is_err());
    let catalog = db.catalog().await.unwrap();
    assert!(
        catalog
            .categories
            .iter()
            .find(|value| value.id == "category-vegetable")
            .unwrap()
            .retired
    );
    assert!(
        catalog
            .sections
            .iter()
            .find(|value| value.id == "section-produce")
            .unwrap()
            .retired
    );
    db.close().await;
}

#[tokio::test]
async fn registry_distinguishes_variants_and_never_seeds_density_rules() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let catalog = db.catalog().await.unwrap();
    assert_eq!(catalog.units.len(), 36);
    assert_eq!(catalog.categories.len(), 6);
    assert_eq!(catalog.sections.len(), 7);
    for id in ["cup-us", "cup-metric", "cup-imperial"] {
        let unit = catalog.units.iter().find(|unit| unit.id == id).unwrap();
        assert!(unit.label.contains("cup"));
        assert!(unit.factor_numerator.is_some());
    }
    for unit in catalog
        .units
        .iter()
        .filter(|unit| unit.dimension == "count" || unit.dimension == "informal")
    {
        assert!(unit.factor_numerator.is_none());
        assert!(unit.factor_denominator.is_none());
    }
    let conversions: i64 = sqlx::query_scalar("SELECT count(*) FROM ingredient_conversion")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(conversions, 0);
    assert!(sqlx::query("INSERT INTO unit(id,label,symbol,dimension,system,factor_numerator,factor_denominator,offset_numerator,offset_denominator) VALUES ('broken','Broken','?','mass','metric',NULL,1,0,1)").execute(&db.pool).await.is_err());
    db.close().await;
}

#[tokio::test]
async fn dates_and_cooked_event_relationships_are_enforced_in_storage() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let meal = db.create_meal(demo()).await.unwrap();
    for week in ["not-a-date", "2026-10-13", "2025-02-30"] {
        assert!(sqlx::query("INSERT INTO plan_entry(id,week_start,meal_id,servings_numerator,servings_denominator) VALUES ('invalid',?,?,1,1)").bind(week).bind(&meal.id).execute(&db.pool).await.is_err());
    }
    sqlx::query("INSERT INTO plan_entry(id,week_start,meal_id,servings_numerator,servings_denominator) VALUES ('plan','2026-10-12',?,1,1)").bind(&meal.id).execute(&db.pool).await.unwrap();
    assert!(sqlx::query("INSERT INTO meal_cooked_event(id,meal_id,local_date,plan_entry_id) VALUES ('bad',?,'not-a-date','plan')").bind(&meal.id).execute(&db.pool).await.is_err());
    let mut second = demo();
    for line in &mut second.ingredients {
        line.id = format!("other-{}", line.id);
    }
    let second = db.create_meal(second).await.unwrap();
    assert!(sqlx::query("INSERT INTO meal_cooked_event(id,meal_id,local_date,plan_entry_id) VALUES ('wrong',?,'2026-10-12','plan')").bind(&second.id).execute(&db.pool).await.is_err());
    sqlx::query("INSERT INTO meal_cooked_event(id,meal_id,local_date,plan_entry_id) VALUES ('valid',?,'2026-10-12','plan')").bind(&meal.id).execute(&db.pool).await.unwrap();
    assert!(
        sqlx::query("UPDATE plan_entry SET meal_id=? WHERE id='plan'")
            .bind(second.id)
            .execute(&db.pool)
            .await
            .is_err()
    );
    db.close().await;
}

#[tokio::test]
async fn text_is_bound_as_data_and_output_contract_uses_camel_case() {
    let directory = TempDir::new().unwrap();
    let db = Database::open(directory.path()).await.unwrap();
    let name = "Herb'); DROP TABLE meal; --";
    let created = db.create_ingredient(ingredient(name)).await.unwrap();
    assert_eq!(created.data.name, name);
    let payload = serde_json::to_value(created).unwrap();
    assert_eq!(payload["preferredUnitId"], "g");
    assert!(payload.get("data").is_none());
    assert_eq!(db.status().await.unwrap().meal_count, 0);
    db.close().await;
}

#[tokio::test]
async fn seed_upgrade_preserves_existing_meals_fields_and_shopping_edits() {
    let directory = TempDir::new().unwrap();
    let scripts = TempDir::new().unwrap();
    for (name, content) in [
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
    ] {
        std::fs::write(scripts.path().join(name), content).unwrap();
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
    let prior = Database { pool };
    sqlx::query("INSERT INTO unit(id,label,symbol,dimension,system,factor_numerator,factor_denominator,offset_numerator,offset_denominator) VALUES ('legacy-g','Legacy gram','g','mass','metric',1,1,0,1)").execute(&prior.pool).await.unwrap();
    let herb = prior
        .create_ingredient(IngredientInput {
            name: "Household herb".into(),
            category_id: None,
            section_id: None,
            preferred_unit_id: Some("legacy-g".into()),
            aliases: vec!["Our herb".into()],
            season_months: vec![6, 7],
            notes: "Preserve this".into(),
        })
        .await
        .unwrap();
    let definition = prior.create_field(field(FieldKind::Boolean)).await.unwrap();
    let mut input = demo();
    for line in &mut input.ingredients {
        line.ingredient_id = Some(herb.id.clone());
        line.unit_id = Some("legacy-g".into());
    }
    input.custom_fields = vec![MealFieldValue {
        field_id: definition.id.clone(),
        value: Some(FieldValue::Boolean(false)),
    }];
    // Populate the historical schema directly: current repositories require
    // current columns and must not masquerade as older application code.
    sqlx::query("INSERT INTO meal(id,title,normalized_title,description,servings_numerator,servings_denominator,prep_minutes,cook_minutes,instructions,notes,source_url,source_site) VALUES ('legacy-meal',?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&input.title).bind(crate::domain::validation::normalized(&input.title)).bind(&input.description).bind(input.base_servings.numerator).bind(input.base_servings.denominator).bind(input.prep_minutes).bind(input.cook_minutes).bind(&input.instructions).bind(&input.notes).bind(&input.source_url).bind(&input.source_site).execute(&prior.pool).await.unwrap();
    for (index, line) in input.ingredients.iter().enumerate() {
        sqlx::query("INSERT INTO meal_ingredient(id,meal_id,ingredient_id,quantity_numerator,quantity_denominator,unit_id,raw_text,source_convention,note,optional,sort_order) VALUES (?,'legacy-meal',?,?,?,?,?,?,?,?,?)")
            .bind(&line.id).bind(&line.ingredient_id).bind(line.quantity.as_ref().map(|q|q.numerator)).bind(line.quantity.as_ref().map(|q|q.denominator)).bind(&line.unit_id).bind(&line.raw_text).bind(&line.source_convention).bind(&line.note).bind(line.optional).bind(index as i64).execute(&prior.pool).await.unwrap();
    }
    for tag in &input.tags {
        sqlx::query("INSERT INTO meal_tag(meal_id,tag) VALUES ('legacy-meal',?)")
            .bind(tag)
            .execute(&prior.pool)
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO meal_custom_field_value(meal_id,field_id,value_json) VALUES ('legacy-meal',?,?)").bind(&definition.id).bind(serde_json::to_string(&FieldValue::Boolean(false)).unwrap()).execute(&prior.pool).await.unwrap();
    sqlx::query("INSERT INTO shopping_list(id,week_start) VALUES ('legacy-list','2026-10-12')")
        .execute(&prior.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO shopping_item(id,list_id,origin,manual_label,checked,has_override,override_json) VALUES ('legacy-manual','legacy-list','manual','Soap',1,1,'{\"amount\":\"2 packs\"}')").execute(&prior.pool).await.unwrap();
    sqlx::query("INSERT INTO app_settings(key,value_json) VALUES ('display.fractions','true')")
        .execute(&prior.pool)
        .await
        .unwrap();
    prior.close().await;
    let upgraded = Database::open(directory.path()).await.unwrap();
    assert_eq!(upgraded.ingredient(&herb.id).await.unwrap(), herb);
    assert_eq!(upgraded.meal("legacy-meal").await.unwrap().data, input);
    assert_eq!(upgraded.field(&definition.id).await.unwrap(), definition);
    let row =
        sqlx::query("SELECT checked,override_json FROM shopping_item WHERE id='legacy-manual'")
            .fetch_one(&upgraded.pool)
            .await
            .unwrap();
    assert!(row.get::<bool, _>("checked"));
    assert_eq!(
        row.get::<String, _>("override_json"),
        "{\"amount\":\"2 packs\"}"
    );
    let preference: String =
        sqlx::query_scalar("SELECT value_json FROM app_settings WHERE key='display.fractions'")
            .fetch_one(&upgraded.pool)
            .await
            .unwrap();
    assert_eq!(preference, "true");
    upgraded.close().await;
}
