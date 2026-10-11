use pantrypal_lib::{
    domain::measurements::{ConversionRequest, ConversionResult, Mode, Purpose},
    persistence::Database,
};
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
#[tokio::test]
async fn phase_one_upgrade_preserves_edited_catalog_existing_rules_and_preferences() {
    let directory = tempfile::tempdir().unwrap();
    let scripts = tempfile::tempdir().unwrap();
    for (name, content) in [
        (
            "0001_catalog.sql",
            include_str!("../migrations/0001_catalog.sql"),
        ),
        (
            "0002_meals.sql",
            include_str!("../migrations/0002_meals.sql"),
        ),
        (
            "0003_planning_storage.sql",
            include_str!("../migrations/0003_planning_storage.sql"),
        ),
        (
            "0004_seed_catalog.sql",
            include_str!("../migrations/0004_seed_catalog.sql"),
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
    sqlx::query("UPDATE ingredient SET notes='Preserve edited onion' WHERE id='ingredient-onion'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE app_settings SET value_json='\"metric\"' WHERE key='measurement.mode'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO app_settings(key,value_json) VALUES ('display.fractions','false')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO ingredient_conversion(id,ingredient_id,from_unit_id,from_numerator,from_denominator,to_unit_id,to_numerator,to_denominator,approximate,notes) VALUES ('prior-onion-rule','ingredient-onion','piece',1,2,'g',75,1,1,'Medium onion')").execute(&pool).await.unwrap();
    pool.close().await;
    let db = Database::open(directory.path()).await.unwrap();
    assert_eq!(db.status().await.unwrap().schema_version, 8);
    assert_eq!(
        db.ingredient("ingredient-onion").await.unwrap().data.notes,
        "Preserve edited onion"
    );
    let prefs = db.measurement_preferences().await.unwrap();
    assert_eq!(prefs.mode, Mode::Metric);
    assert!(!prefs.fractions);
    let rules = db.conversion_rules("ingredient-onion").await.unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, "prior-onion-rule");
    assert_eq!(rules[0].from_denominator, 2);
    assert!(rules[0].approximate);
    assert!(rules[0].original_from_text.is_empty());
    // Legacy rational pairs are retained; the editor renders their exact pair
    // when the original text did not exist in Phase 1.
    let input = ConversionRequest {
        amount: "2".into(),
        from_unit: "piece".into(),
        to_unit: "g".into(),
        ingredient_id: Some("ingredient-onion".into()),
        purpose: Purpose::Quantity,
    };
    assert!(
        matches!(db.convert_measurement(input).await.unwrap(),ConversionResult::Converted{exact,approximate:true,..} if exact=="300")
    );
    db.close().await;
    let db = Database::open(directory.path()).await.unwrap();
    assert_eq!(db.measurement_preferences().await.unwrap(), prefs);
    assert_eq!(
        db.conversion_rules("ingredient-onion").await.unwrap().len(),
        1
    );
    db.close().await;
}
