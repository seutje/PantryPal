use super::{
    catalog::{confirmation, ensure_reference},
    Database,
};
use crate::{
    domain::{catalog::Ingredient, measurements::rational::parse, measurements::*, validation},
    error::{AppError, AppResult},
};
use uuid::Uuid;

impl Database {
    pub async fn search_ingredients(&self, query: String) -> AppResult<Vec<Ingredient>> {
        validation::text(&query, 120, false)?;
        let query = validation::normalized(&query);
        if query.is_empty() {
            return Ok(vec![]);
        }
        let names: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, name FROM ingredient WHERE retired=0 UNION ALL SELECT a.ingredient_id, a.alias FROM ingredient_alias a JOIN ingredient i ON i.id=a.ingredient_id WHERE i.retired=0",
        ).fetch_all(&self.pool).await?;
        let mut scores = std::collections::BTreeMap::<String, usize>::new();
        for (id, name) in names {
            if let Some(rank) = crate::services::ingredient_search::rank(&query, &name) {
                scores
                    .entry(id)
                    .and_modify(|score| *score = (*score).min(rank))
                    .or_insert(rank);
            }
        }
        let mut ordered: Vec<_> = scores.into_iter().collect();
        ordered.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        let mut results = vec![];
        for (id, _) in ordered.into_iter().take(20) {
            results.push(self.ingredient(&id).await?);
        }
        Ok(results)
    }
    pub async fn measurement_preferences(&self) -> AppResult<MeasurementPreferences> {
        let json: String = sqlx::query_scalar(
            "SELECT value_json FROM app_settings WHERE key='measurement.preferences'",
        )
        .fetch_one(&self.pool)
        .await?;
        let prefs: MeasurementPreferences =
            serde_json::from_str(&json).map_err(|_| AppError::storage())?;
        prefs.validate()?;
        Ok(prefs)
    }
    pub async fn save_measurement_preferences(
        &self,
        input: MeasurementPreferences,
    ) -> AppResult<MeasurementPreferences> {
        input.validate()?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE app_settings SET value_json=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE key='measurement.preferences'")
            .bind(serde_json::to_string(&input).map_err(|_|AppError::storage())?).execute(&mut *tx).await?;
        // Keep the original mode key consistent for older export consumers.
        sqlx::query("UPDATE app_settings SET value_json=?,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE key='measurement.mode'")
            .bind(serde_json::to_string(&input.mode).map_err(|_|AppError::storage())?).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO app_settings(key,value_json) VALUES ('display.fractions',?) ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')")
            .bind(if input.fractions {"true"} else {"false"}).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(input)
    }
    pub async fn conversion_rules(&self, ingredient_id: &str) -> AppResult<Vec<ConversionRule>> {
        validation::id(ingredient_id)?;
        Ok(sqlx::query_as(
            "SELECT * FROM ingredient_conversion WHERE ingredient_id=? AND retired=0 ORDER BY id",
        )
        .bind(ingredient_id)
        .fetch_all(&self.pool)
        .await?)
    }
    pub async fn save_conversion_rule(
        &self,
        id: Option<String>,
        input: RuleInput,
    ) -> AppResult<ConversionRule> {
        validation::optional_id(&id)?;
        validation::id(&input.ingredient_id)?;
        validation::id(&input.from_unit_id)?;
        validation::id(&input.to_unit_id)?;
        validation::text(&input.notes, 10_000, true)?;
        let positive = |raw: &str| -> AppResult<(i64, i64)> {
            let x = parse(raw)?.ok_or_else(|| {
                AppError::validation("Conversion rules require known positive amounts.")
            })?;
            if x.n <= 0 {
                return Err(AppError::validation(
                    "Conversion rules require positive amounts.",
                ));
            }
            Ok((x.n as i64, x.d as i64))
        };
        let (from_n, from_d) = positive(&input.from_amount)?;
        let (to_n, to_d) = positive(&input.to_amount)?;
        let mut tx = self.pool.begin().await?;
        ensure_reference(
            &mut tx,
            "SELECT count(*) FROM ingredient WHERE id=? AND retired=0",
            &input.ingredient_id,
        )
        .await?;
        let from: crate::domain::catalog::Unit = sqlx::query_as("SELECT * FROM unit WHERE id=?")
            .bind(&input.from_unit_id)
            .fetch_one(&mut *tx)
            .await?;
        let to: crate::domain::catalog::Unit = sqlx::query_as("SELECT * FROM unit WHERE id=?")
            .bind(&input.to_unit_id)
            .fetch_one(&mut *tx)
            .await?;
        if from.dimension == "temperature"
            || to.dimension == "temperature"
            || from.dimension == "informal"
            || to.dimension == "informal"
            || compatible(&from, &to)
        {
            return Err(AppError::validation("Use ingredient rules only between distinct count units or different quantity dimensions."));
        }
        let editing = id.is_some();
        let rule_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let existing: Option<(String, bool)> =
            sqlx::query_as("SELECT ingredient_id,retired FROM ingredient_conversion WHERE id=?")
                .bind(&rule_id)
                .fetch_optional(&mut *tx)
                .await?;
        if editing && existing.is_none() {
            return Err(AppError::not_found());
        }
        if existing
            .as_ref()
            .is_some_and(|(ingredient, retired)| ingredient != &input.ingredient_id || *retired)
        {
            return Err(AppError::conflict());
        }
        let duplicate: i64 = sqlx::query_scalar("SELECT count(*) FROM ingredient_conversion WHERE ingredient_id=? AND retired=0 AND id<>? AND ((from_unit_id=? AND to_unit_id=?) OR (from_unit_id=? AND to_unit_id=?))")
            .bind(&input.ingredient_id).bind(&rule_id).bind(&input.from_unit_id).bind(&input.to_unit_id).bind(&input.to_unit_id).bind(&input.from_unit_id).fetch_one(&mut *tx).await?;
        if duplicate != 0 {
            return Err(AppError::validation(
                "A rule for this unit pair already exists. Edit the existing rule.",
            ));
        }
        sqlx::query("INSERT INTO ingredient_conversion(id,ingredient_id,from_unit_id,from_numerator,from_denominator,to_unit_id,to_numerator,to_denominator,approximate,notes,original_from_text,original_to_text) VALUES (?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET from_unit_id=excluded.from_unit_id,from_numerator=excluded.from_numerator,from_denominator=excluded.from_denominator,to_unit_id=excluded.to_unit_id,to_numerator=excluded.to_numerator,to_denominator=excluded.to_denominator,approximate=excluded.approximate,notes=excluded.notes,original_from_text=excluded.original_from_text,original_to_text=excluded.original_to_text")
            .bind(&rule_id).bind(&input.ingredient_id).bind(&input.from_unit_id).bind(from_n).bind(from_d).bind(&input.to_unit_id).bind(to_n).bind(to_d).bind(input.approximate).bind(&input.notes).bind(&input.from_amount).bind(&input.to_amount).execute(&mut *tx).await?;
        let rule = sqlx::query_as("SELECT * FROM ingredient_conversion WHERE id=?")
            .bind(&rule_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(rule)
    }
    pub async fn retire_conversion_rule(&self, id: &str, confirmed: bool) -> AppResult<()> {
        validation::id(id)?;
        confirmation(confirmed)?;
        if sqlx::query("UPDATE ingredient_conversion SET retired=1 WHERE id=?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected()
            == 0
        {
            return Err(AppError::not_found());
        }
        Ok(())
    }
    pub async fn convert_measurement(
        &self,
        input: ConversionRequest,
    ) -> AppResult<ConversionResult> {
        validation::optional_id(&input.ingredient_id)?;
        if let Some(id) = &input.ingredient_id {
            self.ingredient(id).await?;
        }
        let units = sqlx::query_as("SELECT * FROM unit ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        let aliases =
            sqlx::query_as("SELECT normalized_alias,unit_id FROM unit_alias ORDER BY unit_id")
                .fetch_all(&self.pool)
                .await?;
        let rules = if let Some(id) = &input.ingredient_id {
            self.conversion_rules(id).await?
        } else {
            vec![]
        };
        convert(
            input,
            &units,
            &aliases,
            &rules,
            &self.measurement_preferences().await?,
        )
    }
    /// Exact normalized identity matches include retired records so reserved
    /// historical names are visible. No fuzzy match is automatically accepted.
    pub async fn ingredient_candidates(
        &self,
        names: Vec<String>,
        exclude_id: Option<String>,
    ) -> AppResult<Vec<Ingredient>> {
        validation::optional_id(&exclude_id)?;
        if names.len() > 51 {
            return Err(AppError::validation("Too many names."));
        }
        let mut ids = std::collections::BTreeSet::new();
        for name in names {
            validation::text(&name, 120, false)?;
            let name = validation::normalized(&name);
            if name.is_empty() {
                continue;
            }
            let matches: Vec<String> = sqlx::query_scalar("SELECT id FROM ingredient WHERE normalized_name=? UNION SELECT ingredient_id FROM ingredient_alias WHERE normalized_alias=?")
                .bind(&name).bind(&name).fetch_all(&self.pool).await?;
            for id in matches {
                if Some(&id) != exclude_id.as_ref() {
                    ids.insert(id);
                }
            }
        }
        let mut result = vec![];
        for id in ids {
            result.push(self.ingredient(&id).await?);
        }
        Ok(result)
    }
}
