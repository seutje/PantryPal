use pantrypal_lib::persistence::{Database, MIGRATOR};
use sqlx::{sqlite::SqliteConnectOptions, Connection};
#[tokio::test]
async fn schema_seven_upgrade_retains_all_shopping_rows_and_original_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let mut connection = sqlx::SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(dir.path().join("pantrypal.sqlite3"))
            .create_if_missing(true)
            .foreign_keys(true),
    )
    .await
    .unwrap();
    let migrations: Vec<_> = MIGRATOR
        .iter()
        .filter(|m| m.version <= 7)
        .cloned()
        .collect();
    let old = sqlx::migrate::Migrator {
        migrations: std::borrow::Cow::Owned(migrations),
        ..MIGRATOR
    };
    old.run(&mut connection).await.unwrap();
    sqlx::query("INSERT INTO shopping_list(id,week_start,generation_revision,generation_metadata_json) VALUES ('old','2020-12-28',9,'{\"source\":\"original\"}')").execute(&mut connection).await.unwrap();
    sqlx::query("INSERT INTO shopping_item(id,list_id,ingredient_id,origin,generation_key,generated_basis_json,has_override,override_json,unit_id,checked,excluded,section_id) VALUES ('edited','old','ingredient-rice','generated','legacy-key','{\"amount\":4}',1,'{\"amount\":2}','cup-us',1,1,'section-pantry')").execute(&mut connection).await.unwrap();
    connection.close().await.unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    assert_eq!(db.status().await.unwrap().schema_version, 8);
    let view = db.get_shopping("2020-12-28").await.unwrap();
    assert_eq!(view.items.len(), 1);
    let item = &view.items[0].item;
    assert!(item.checked && item.excluded);
    assert_eq!(item.unit_id.as_deref(), Some("cup-us"));
    assert_eq!(item.legacy.as_ref().unwrap()["generationKey"], "legacy-key");
    assert_eq!(item.legacy.as_ref().unwrap()["basis"], "{\"amount\":4}");
    db.close().await;
    let mut connection = sqlx::SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(dir.path().join("pantrypal.sqlite3"))
            .read_only(true),
    )
    .await
    .unwrap();
    let metadata: String =
        sqlx::query_scalar("SELECT generation_metadata_json FROM shopping_list WHERE id='old'")
            .fetch_one(&mut connection)
            .await
            .unwrap();
    assert_eq!(metadata, "{\"source\":\"original\"}");
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM shopping_item WHERE id='edited' AND checked=1 AND excluded=1 AND has_override=1").fetch_one(&mut connection).await.unwrap();
    assert_eq!(count, 1);
    connection.close().await.unwrap();
}
