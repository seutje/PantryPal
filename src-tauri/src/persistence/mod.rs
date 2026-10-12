//! Parameterized repositories. The frontend never supplies SQL or a path.
mod backup_format;
mod backup_validation;
pub mod backups;
pub mod catalog;
pub mod fields;
mod history;
pub mod imports;
pub mod meals;
pub mod measurements;
mod planning;
mod shopping;
pub mod usability;

use crate::error::{AppError, AppResult};
use serde::Serialize;
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::{path::Path, time::Duration};

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct Database {
    pub(crate) pool: SqlitePool,
    pub(crate) app_data: std::path::PathBuf,
}

/// One managed instance per application. Initialization errors remain visible
/// in the shell; they never trigger a fallback or recreation of the database.
pub struct StorageState(pub AppResult<Database>);

impl StorageState {
    pub fn database(&self) -> AppResult<&Database> {
        self.0.as_ref().map_err(Clone::clone)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageStatus {
    pub schema_version: i64,
    pub ingredient_count: i64,
    pub meal_count: i64,
}

impl Database {
    /// Backend-owned directory only. Tauri resolves app_data_dir at startup;
    /// tests supply isolated temporary directories. No IPC accepts this path.
    pub async fn open(app_data: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(app_data).map_err(|_| AppError::storage())?;
        let options = SqliteConnectOptions::new()
            .filename(app_data.join("pantrypal.sqlite3"))
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_secs(5));
        // A single connection serializes ordinary household writes and avoids
        // read snapshots mixing revisions while assembling multi-row records.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        if app_data
            .join("pantrypal.sqlite3")
            .metadata()
            .is_ok_and(|m| m.len() > 0)
        {
            let exists: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM sqlite_master WHERE name='_sqlx_migrations'",
            )
            .fetch_one(&pool)
            .await?;
            if exists != 0 {
                let version: i64 = sqlx::query_scalar(
                    "SELECT coalesce(max(version),0) FROM _sqlx_migrations WHERE success=1",
                )
                .fetch_one(&pool)
                .await?;
                if version > 0 && version < backups::schema_version() {
                    let folder = app_data.join("backups");
                    std::fs::create_dir_all(&folder).map_err(|_| AppError::storage())?;
                    let path =
                        folder.join(format!("before-migration-{}.sqlite3", uuid::Uuid::new_v4()));
                    sqlx::query("VACUUM INTO ?")
                        .bind(path.to_string_lossy().as_ref())
                        .execute(&pool)
                        .await?;
                }
            }
        }
        if MIGRATOR.run(&pool).await.is_err() {
            pool.close().await;
            return Err(AppError { code: "migration_failed", message: "Could not open this data version. Use a compatible PantryPal version; your data has not been reset.".into() });
        }
        Ok(Self {
            pool,
            app_data: app_data.into(),
        })
    }

    pub async fn status(&self) -> AppResult<StorageStatus> {
        let mut tx = self.pool.begin().await?;
        let schema_version = sqlx::query_scalar(
            "SELECT coalesce(max(version),0) FROM _sqlx_migrations WHERE success=1",
        )
        .fetch_one(&mut *tx)
        .await?;
        let ingredient_count =
            sqlx::query_scalar("SELECT count(*) FROM ingredient WHERE retired=0")
                .fetch_one(&mut *tx)
                .await?;
        let meal_count = sqlx::query_scalar("SELECT count(*) FROM meal WHERE retired=0")
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(StorageStatus {
            schema_version,
            ingredient_count,
            meal_count,
        })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod backup_tests;
#[cfg(test)]
mod meal_library_tests;
#[cfg(test)]
mod measurement_tests;
#[cfg(test)]
mod planning_tests;
#[cfg(test)]
mod shopping_tests;
#[cfg(test)]
mod tests;
