//! Parameterized repositories. The frontend never supplies SQL or a path.
pub mod catalog;
pub mod fields;
mod history;
pub mod meals;
pub mod measurements;

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
        if MIGRATOR.run(&pool).await.is_err() {
            pool.close().await;
            return Err(AppError { code: "migration_failed", message: "Could not open this data version. Use a compatible PantryPal version; your data has not been reset.".into() });
        }
        Ok(Self { pool })
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
mod meal_library_tests;
#[cfg(test)]
mod measurement_tests;
#[cfg(test)]
mod tests;
