use super::{backup_format as format, Database, MIGRATOR};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{
    hash::{Hash, Hasher},
    io::Write,
};

pub fn schema_version() -> i64 {
    MIGRATOR.iter().last().expect("embedded migrations").version
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreMode {
    Merge,
    Replace,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub directory: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreview {
    pub expected_state: String,
    pub meals: usize,
    pub ingredients: usize,
    pub rows: usize,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub path: String,
}
fn state(data: &format::Snapshot) -> AppResult<String> {
    let text = serde_json::to_string(data).map_err(|_| AppError::storage())?;
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hash);
    Ok(format!("{:016x}", hash.finish()))
}
impl Database {
    pub fn backup_info(&self) -> BackupInfo {
        BackupInfo {
            directory: self.app_data.join("backups").to_string_lossy().into(),
        }
    }
    fn write_backup(&self, data: &format::Snapshot) -> AppResult<BackupResult> {
        if data.tables.iter().map(|t| t.rows.len()).sum::<usize>() > 100_000 {
            return Err(AppError::validation(
                "Backup exceeds 100,000 rows. No data was changed.",
            ));
        }
        let text = serde_json::to_vec_pretty(data).map_err(|_| AppError::storage())?;
        if text.len() > format::LIMIT {
            return Err(AppError::validation(
                "Backup exceeds 50 MiB. No data was changed.",
            ));
        }
        let directory = self.app_data.join("backups");
        std::fs::create_dir_all(&directory).map_err(|_| AppError::storage())?;
        let name = format!("pantrypal-{}.json", uuid::Uuid::new_v4());
        let final_path = directory.join(name);
        let temporary = final_path.with_extension("partial");
        let result = (|| {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(&text)?;
            file.sync_all()?;
            std::fs::rename(&temporary, &final_path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
            return Err(AppError::storage());
        }
        Ok(BackupResult {
            path: final_path.to_string_lossy().into(),
        })
    }
    pub async fn export_backup(&self) -> AppResult<BackupResult> {
        let mut tx = self.pool.begin().await?;
        let data = format::snapshot(&mut tx).await?;
        let result = self.write_backup(&data)?;
        tx.commit().await?;
        Ok(result)
    }
    async fn prepare_restore(
        &self,
        text: &str,
        mode: RestoreMode,
        current: format::Snapshot,
    ) -> AppResult<format::Snapshot> {
        let mut connection = self.pool.acquire().await?;
        let shapes = format::shapes(&mut connection).await?;
        drop(connection);
        let incoming = format::parse(text, &shapes)?;
        // A migrated isolated in-memory database applies all real constraints and
        // triggers before any household write. Its errors contain no payload.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .in_memory(true)
                    .foreign_keys(true),
            )
            .await?;
        let stage = Database {
            pool,
            app_data: self.app_data.clone(),
        };
        MIGRATOR
            .run(&stage.pool)
            .await
            .map_err(|_| AppError::storage())?;
        let result = async {
            let canonical={let mut c=stage.pool.acquire().await?; format::snapshot(&mut c).await?};
            for name in ["unit","unit_alias"] {
                let incoming=incoming.tables.iter().find(|t|t.name==name).ok_or_else(AppError::storage)?;
                let canonical=canonical.tables.iter().find(|t|t.name==name).ok_or_else(AppError::storage)?;
                if canonical.rows.iter().any(|row| !incoming.rows.contains(row)) {
                    return Err(AppError::validation("Backup unit registry does not match the supported measurement definitions."));
                }
            }
            let mut tx=stage.pool.begin().await?;
            format::replace(&mut tx,&incoming).await?;
            tx.commit().await?;
            stage.validate_backup().await?;
            let combined = match mode { RestoreMode::Replace => incoming, RestoreMode::Merge => format::merge(current,&incoming,&shapes)? };
            let mut tx=stage.pool.begin().await?;
            format::replace(&mut tx,&combined).await?;
            tx.commit().await?;
            stage.validate_backup().await?;
            Ok(combined)
        }.await;
        stage.close().await;
        result.map_err(|_: AppError| {
            AppError::validation("Backup validation or merge failed. Check its version, references, values and conflicting records. Your data has not been changed.")
        })
    }
    pub async fn preview_restore(
        &self,
        text: &str,
        mode: RestoreMode,
    ) -> AppResult<RestorePreview> {
        let current = {
            let mut c = self.pool.acquire().await?;
            format::snapshot(&mut c).await?
        };
        let expected_state = state(&current)?;
        let combined = self.prepare_restore(text, mode, current).await?;
        let rows = combined.tables.iter().map(|t| t.rows.len()).sum();
        let count = |name| {
            combined
                .tables
                .iter()
                .find(|t| t.name == name)
                .map_or(0, |t| t.rows.len())
        };
        Ok(RestorePreview {
            expected_state,
            meals: count("meal"),
            ingredients: count("ingredient"),
            rows,
        })
    }
    pub async fn restore_backup(
        &self,
        text: &str,
        mode: RestoreMode,
        expected_state: &str,
        confirmed: bool,
    ) -> AppResult<BackupResult> {
        if !confirmed {
            return Err(AppError::validation(
                "Confirm the reviewed merge or replacement first.",
            ));
        }
        let current = {
            let mut c = self.pool.acquire().await?;
            format::snapshot(&mut c).await?
        };
        if state(&current)? != expected_state {
            return Err(AppError::conflict());
        }
        let combined = self.prepare_restore(text, mode, current).await?;
        let mut tx = self.pool.begin().await?;
        let current = format::snapshot(&mut tx).await?;
        if state(&current)? != expected_state {
            return Err(AppError::conflict());
        }
        let recovery = self.write_backup(&current)?; // Fail closed before any deletion.
        format::replace(&mut tx, &combined).await?;
        tx.commit().await?;
        Ok(recovery)
    }
}
