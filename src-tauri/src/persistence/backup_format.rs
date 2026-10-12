//! Versioned logical snapshots. SQL names come exclusively from the compiled
//! table registry and the local migration schema, never from imported content.
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Row, SqliteConnection, TypeInfo, ValueRef};
use std::collections::HashMap;

pub(super) const LIMIT: usize = 50 * 1024 * 1024;
pub(super) const TABLES: &[&str] = &[
    "unit",
    "unit_alias",
    "ingredient_category",
    "shopping_section",
    "ingredient",
    "ingredient_alias",
    "ingredient_season_month",
    "ingredient_conversion",
    "custom_field_definition",
    "meal",
    "meal_tag",
    "meal_ingredient",
    "meal_custom_field_value",
    "plan_week",
    "plan_entry",
    "meal_cooked_event",
    "cooked_event_change",
    "planner_action",
    "shopping_list",
    "shopping_item",
    "import_record",
    "app_settings",
];
#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Snapshot {
    pub format: String,
    pub format_version: u8,
    pub schema_version: i64,
    pub tables: Vec<Table>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}
pub(super) struct Shape {
    pub columns: Vec<String>,
    pub types: Vec<String>,
    pub keys: Vec<usize>,
}
pub(super) async fn shapes(c: &mut SqliteConnection) -> AppResult<Vec<Shape>> {
    let mut shapes = vec![];
    for table in TABLES {
        let rows = sqlx::query(&format!("PRAGMA table_info({table})"))
            .fetch_all(&mut *c)
            .await?;
        let columns = rows
            .iter()
            .map(|r| r.try_get("name"))
            .collect::<Result<_, _>>()?;
        let mut keys: Vec<(i64, usize)> = rows
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                let pk: i64 = r.get("pk");
                (pk > 0).then_some((pk, i))
            })
            .collect();
        keys.sort();
        shapes.push(Shape {
            columns,
            types: rows.iter().map(|r| r.get("type")).collect(),
            keys: keys.into_iter().map(|(_, i)| i).collect(),
        });
    }
    Ok(shapes)
}
pub(super) async fn snapshot(c: &mut SqliteConnection) -> AppResult<Snapshot> {
    let schema = shapes(c).await?;
    let mut tables = vec![];
    for (name, shape) in TABLES.iter().zip(schema) {
        let order = shape
            .keys
            .iter()
            .map(|i| shape.columns[*i].as_str())
            .collect::<Vec<_>>()
            .join(",");
        let rows = sqlx::query(&format!("SELECT * FROM {name} ORDER BY {order}"))
            .fetch_all(&mut *c)
            .await?;
        let mut values = vec![];
        for row in rows {
            let mut cells = vec![];
            for i in 0..shape.columns.len() {
                let raw = row.try_get_raw(i)?;
                cells.push(if raw.is_null() {
                    Value::Null
                } else {
                    match raw.type_info().name() {
                        "INTEGER" => Value::from(row.try_get::<i64, _>(i)?),
                        "REAL" => Value::from(row.try_get::<f64, _>(i)?),
                        "TEXT" => Value::from(row.try_get::<String, _>(i)?),
                        _ => return Err(AppError::storage()),
                    }
                });
            }
            values.push(cells);
        }
        tables.push(Table {
            name: (*name).into(),
            columns: shape.columns,
            rows: values,
        });
    }
    Ok(Snapshot {
        format: "PantryPal".into(),
        format_version: 1,
        schema_version: super::backups::schema_version(),
        tables,
    })
}
pub(super) fn parse(text: &str, shapes: &[Shape]) -> AppResult<Snapshot> {
    if text.len() > LIMIT {
        return Err(AppError::validation("Backup exceeds the 50 MiB limit."));
    }
    let data: Snapshot = serde_json::from_str(text)
        .map_err(|_| AppError::validation("This is not a valid PantryPal JSON backup."))?;
    if data.format != "PantryPal"
        || data.format_version != 1
        || data.schema_version != super::backups::schema_version()
    {
        return Err(AppError::validation("Unsupported backup version. Use the matching PantryPal version to export compatible data."));
    }
    if data.tables.len() != TABLES.len() {
        return Err(AppError::validation("Backup tables are incomplete."));
    }
    let mut total = 0;
    for ((table, name), shape) in data.tables.iter().zip(TABLES).zip(shapes) {
        if table.name != *name || table.columns != shape.columns {
            return Err(AppError::validation(
                "Backup schema does not match this version.",
            ));
        }
        total += table.rows.len();
        if total > 100_000
            || table.rows.iter().any(|r| {
                r.len() != shape.columns.len()
                    || r.iter()
                        .zip(&shape.types)
                        .any(|(v, kind)| !match (v, kind.as_str()) {
                            (Value::Null, _) => true,
                            (Value::String(_), "TEXT") => true,
                            (Value::Number(n), "INTEGER") => n.is_i64(),
                            (Value::Number(n), "REAL") => n.is_i64() || n.is_f64(),
                            _ => false,
                        })
            })
        {
            return Err(AppError::validation(
                "Backup rows are invalid or exceed the supported limit.",
            ));
        }
    }
    Ok(data)
}
pub(super) async fn replace(c: &mut SqliteConnection, data: &Snapshot) -> AppResult<()> {
    for name in TABLES.iter().rev() {
        sqlx::query(&format!("DELETE FROM {name}"))
            .execute(&mut *c)
            .await?;
    }
    for table in &data.tables {
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table.name,
            table.columns.join(","),
            vec!["?"; table.columns.len()].join(",")
        );
        for row in &table.rows {
            let mut query = sqlx::query(&sql);
            for value in row {
                query = match value {
                    Value::Null => query.bind(Option::<String>::None),
                    Value::String(text) => query.bind(text),
                    Value::Number(n) if n.is_i64() => query.bind(n.as_i64().unwrap()),
                    Value::Number(n) if n.is_f64() => query.bind(
                        n.as_f64()
                            .ok_or_else(|| AppError::validation("Invalid number in backup."))?,
                    ),
                    _ => return Err(AppError::validation("Invalid backup cell.")),
                };
            }
            query.execute(&mut *c).await?;
        }
    }
    Ok(())
}
pub(super) fn merge(
    mut current: Snapshot,
    incoming: &Snapshot,
    shapes: &[Shape],
) -> AppResult<Snapshot> {
    for ((table, other), shape) in current.tables.iter_mut().zip(&incoming.tables).zip(shapes) {
        let key = |row: &Vec<Value>| {
            shape
                .keys
                .iter()
                .map(|i| row[*i].to_string())
                .collect::<Vec<_>>()
                .join("\0")
        };
        let mut keys: HashMap<_, _> = table
            .rows
            .iter()
            .enumerate()
            .map(|(i, r)| (key(r), i))
            .collect();
        for row in &other.rows {
            let identity = key(row);
            if let Some(i) = keys.get(&identity) {
                if table.rows[*i]
                    .iter()
                    .zip(row)
                    .zip(&table.columns)
                    .any(|((old, new), column)| {
                        !matches!(column.as_str(), "created_at" | "updated_at") && old != new
                    })
                {
                    return Err(AppError::validation("Merge found different data for an existing ID. No records were changed. Use Replace only if you intend to restore the entire backup."));
                }
            } else {
                keys.insert(identity, table.rows.len());
                table.rows.push(row.clone());
            }
        }
    }
    Ok(current)
}
