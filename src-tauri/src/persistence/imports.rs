use super::{
    meals::{insert_meal, load_meal},
    planning::{receipt, replay},
    Database,
};
use crate::{
    domain::{meal_library::MealDraft, meals::Meal},
    error::{AppError, AppResult},
    importers::{self, ImportInput, ImportedRecipeDraft},
    services::meal_library::draft_input,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmImport {
    pub action_id: String,
    pub source: ImportInput,
    pub reviewed: MealDraft,
    pub confirmed: bool,
    pub allow_duplicate: bool,
}
impl Database {
    pub async fn preview_import(&self, input: &ImportInput) -> AppResult<ImportedRecipeDraft> {
        let mut draft = importers::extract(input)?;
        let mode = self.measurement_preferences().await?.mode;
        for line in &mut draft.meal.ingredients {
            let parsed = importers::parse_site_line(
                &line.raw_text,
                &mode,
                draft.meal.source_site.as_deref(),
            )?;
            line.unit_id = parsed.unit_id;
            line.source_convention = parsed.source_convention;
        }
        draft.warnings.push(if mode == crate::domain::measurements::Mode::Metric {
            "Unqualified cups and spoons use metric variants; other regional units default to US. Explicit source units are preserved."
        } else {
            "Unqualified regional units use US variants. Explicit source units are preserved."
        }.into());
        let catalog = self.catalog().await?;
        for line in &mut draft.meal.ingredients {
            let name = line
                .note
                .strip_prefix("Review ingredient: ")
                .unwrap_or(&line.raw_text)
                .lines()
                .next()
                .unwrap_or_default();
            let name = crate::domain::validation::normalized(name);
            let matches: Vec<_> = catalog
                .ingredients
                .iter()
                .filter(|i| {
                    !i.retired
                        && std::iter::once(&i.data.name)
                            .chain(i.data.aliases.iter())
                            .any(|s| crate::domain::validation::normalized(s) == name)
                })
                .collect();
            if matches.len() == 1 {
                line.ingredient_id = Some(matches[0].id.clone());
                // Keep site section/preparation notes after resolving the name.
                line.note = line
                    .note
                    .split_once('\n')
                    .map(|(_, rest)| rest.to_owned())
                    .unwrap_or_default();
            }
        }
        draft.duplicate_titles=sqlx::query_scalar("SELECT DISTINCT m.title FROM meal m LEFT JOIN import_record i ON i.meal_id=m.id WHERE m.source_url=? OR i.source_url=? ORDER BY m.title")
            .bind(&draft.meal.source_url).bind(&draft.meal.source_url).fetch_all(&self.pool).await?;
        Ok(draft)
    }
    pub async fn confirm_import(&self, request: ConfirmImport) -> AppResult<Meal> {
        if !request.confirmed {
            return Err(AppError::validation(
                "Review the recipe and confirm saving.",
            ));
        }
        let original = importers::extract(&request.source)?;
        if request.reviewed.source_url != original.meal.source_url
            || request.reviewed.source_site != original.meal.source_site
        {
            return Err(AppError::validation(
                "Keep the reviewed import source unchanged. Edit it after saving if needed.",
            ));
        }
        let input = draft_input(request.reviewed.clone(), None)?;
        input.validate()?;
        let encoded = serde_json::to_string(
            &serde_json::json!({"scope":"recipe_import_v1","request":request}),
        )
        .map_err(|_| AppError::storage())?;
        let mut tx = self.pool.begin().await?;
        if replay(&mut tx, &request.action_id, &encoded).await? {
            let id: String = sqlx::query_scalar("SELECT meal_id FROM import_record WHERE id=?")
                .bind(&request.action_id)
                .fetch_one(&mut *tx)
                .await?;
            let meal = load_meal(&mut tx, &id).await?;
            tx.commit().await?;
            return Ok(meal);
        }
        let duplicate:i64=sqlx::query_scalar("SELECT count(*) FROM meal m LEFT JOIN import_record i ON i.meal_id=m.id WHERE m.source_url=? OR i.source_url=?").bind(&input.source_url).bind(&input.source_url).fetch_one(&mut *tx).await?;
        if duplicate > 0 && !request.allow_duplicate {
            return Err(AppError::validation(
                "This source was already imported. Confirm saving another meal or cancel.",
            ));
        }
        let meal = insert_meal(&mut tx, &input).await?;
        let provenance=serde_json::to_string(&serde_json::json!({"version":1,"format":request.source.format,"originalPayload":request.source.payload,"originalYield":original.original_yield,"attribution":original.attribution,"reviewed":request.reviewed})).map_err(|_|AppError::storage())?;
        sqlx::query("INSERT INTO import_record(id,meal_id,source_url,source_site,provenance_json) VALUES (?,?,?,?,?)").bind(&request.action_id).bind(&meal.id).bind(&input.source_url).bind(&input.source_site).bind(provenance).execute(&mut *tx).await?;
        receipt(&mut tx, &request.action_id, &encoded).await?;
        tx.commit().await?;
        Ok(meal)
    }
}
