use super::{
    catalog::load_ingredient,
    meals::load_meal,
    planning::{load_entry, receipt, replay},
    Database,
};
use crate::{
    domain::{planning::validate_week, shopping::*, validation},
    error::{AppError, AppResult},
};
use serde_json::json;
use sqlx::{Row, Sqlite, Transaction};

async fn context(tx: &mut Transaction<'_, Sqlite>, week: &str) -> AppResult<(Context, String)> {
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM plan_entry WHERE week_start=? ORDER BY id")
            .bind(week)
            .fetch_all(&mut **tx)
            .await?;
    let mut entries = vec![];
    let mut meals = vec![];
    for id in ids {
        let entry = load_entry(tx, &id).await?;
        if !meals
            .iter()
            .any(|m: &crate::domain::meals::Meal| m.id == entry.meal_id)
        {
            meals.push(load_meal(tx, &entry.meal_id).await?);
        }
        entries.push(entry);
    }
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM ingredient ORDER BY id")
        .fetch_all(&mut **tx)
        .await?;
    let mut ingredients = vec![];
    for id in ids {
        ingredients.push(load_ingredient(tx, &id).await?);
    }
    let units = sqlx::query_as("SELECT * FROM unit ORDER BY id")
        .fetch_all(&mut **tx)
        .await?;
    let sections=sqlx::query_as("SELECT id,name,sort_order,retired FROM shopping_section ORDER BY sort_order,normalized_name,id").fetch_all(&mut **tx).await?;
    let rules = sqlx::query_as("SELECT * FROM ingredient_conversion WHERE retired=0 ORDER BY id")
        .fetch_all(&mut **tx)
        .await?;
    let prefs: String = sqlx::query_scalar(
        "SELECT value_json FROM app_settings WHERE key='measurement.preferences'",
    )
    .fetch_one(&mut **tx)
    .await?;
    let ctx = Context {
        entries,
        meals,
        ingredients,
        units,
        sections,
        rules,
        prefs: serde_json::from_str(&prefs).map_err(|_| AppError::storage())?,
    };
    ctx.prefs.validate()?;
    // Semantic fingerprint includes source pairs/text, identities, rules, units,
    // section ordering and preferences. Read and write use the same transaction.
    let fingerprint=json!({"version":1,"entries":ctx.entries,"meals":ctx.meals,"ingredients":ctx.ingredients,"units":ctx.units,"sections":ctx.sections,"rules":ctx.rules,"prefs":ctx.prefs}).to_string();
    Ok((ctx, fingerprint))
}
async fn document(
    tx: &mut Transaction<'_, Sqlite>,
    week: &str,
    ctx: &Context,
) -> AppResult<(Document, i64, bool, String)> {
    let row=sqlx::query("SELECT id,revision,generation_revision,generation_metadata_json,document_json FROM shopping_list WHERE week_start=?").bind(week).fetch_optional(&mut **tx).await?;
    let Some(row) = row else {
        return Ok((Document::default(), 0, false, String::new()));
    };
    let revision = row.try_get("revision")?;
    let generated = row.try_get::<i64, _>("generation_revision")? > 0;
    let fingerprint = row.try_get("generation_metadata_json")?;
    if let Some(raw) = row.try_get::<Option<String>, _>("document_json")? {
        let document: Document = serde_json::from_str(&raw).map_err(|_| AppError::storage())?;
        if document.version != 1 {
            return Err(AppError::validation(
                "This shopping document requires a newer PantryPal version.",
            ));
        }
        return Ok((document, revision, generated, fingerprint));
    }
    // Pre-engine rows had no standardized basis/override payload. Carry every
    // field forward verbatim, display its serialized adjustment for review,
    // and keep it as a manual carry-over instead of guessing its semantics.
    let legacy = sqlx::query("SELECT * FROM shopping_item WHERE list_id=? ORDER BY sort_order,id")
        .bind(row.try_get::<String, _>("id")?)
        .fetch_all(&mut **tx)
        .await?;
    let mut doc = Document::default();
    for row in legacy {
        let ingredient_id: Option<String> = row.try_get("ingredient_id")?;
        let label = row
            .try_get::<Option<String>, _>("manual_label")?
            .or_else(|| {
                ingredient_id.as_ref().and_then(|id| {
                    ctx.ingredients
                        .iter()
                        .find(|i| &i.id == id)
                        .map(|i| i.data.name.clone())
                })
            })
            .unwrap_or_else(|| "Legacy item".into());
        let override_json: Option<String> = row.try_get("override_json")?;
        let provenance = json!({"origin":row.try_get::<String,_>("origin")?,"generationKey":row.try_get::<Option<String>,_>("generation_key")?,"basis":row.try_get::<Option<String>,_>("generated_basis_json")?,"hasOverride":row.try_get::<bool,_>("has_override")?,"override":override_json,"sortOrder":row.try_get::<i64,_>("sort_order")?});
        doc.items.push(ShoppingItem {
            id: row.try_get("id")?,
            key: None,
            ingredient_id,
            label,
            section_id: row.try_get("section_id")?,
            numerator: None,
            denominator: None,
            unit_id: row.try_get("unit_id")?,
            amount_text: override_json.unwrap_or_else(|| "Unknown legacy quantity".into()),
            approximate: false,
            warnings: vec![
                "Legacy carry-over: review its original quantity before shopping.".into(),
            ],
            rule_ids: vec![],
            sources: vec![],
            adjustment: None,
            checked: row.try_get("checked")?,
            excluded: row.try_get("excluded")?,
            orphaned: false,
            legacy: Some(provenance),
        });
    }
    Ok((doc, revision, generated, fingerprint))
}
impl Database {
    pub async fn get_shopping(&self, week: &str) -> AppResult<ShoppingView> {
        validate_week(week)?;
        let mut tx = self.pool.begin().await?;
        let (ctx, fingerprint) = context(&mut tx, week).await?;
        let (doc, revision, generated, old) = document(&mut tx, week, &ctx).await?;
        let result = view(
            week.into(),
            revision,
            generated,
            generated && old != fingerprint,
            doc,
            &ctx,
        )?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn change_shopping(&self, input: ShoppingMutation) -> AppResult<ShoppingView> {
        validate_week(&input.week_start)?;
        validation::id(&input.action_id)?;
        let request = json!({"scope":"shopping","input":input}).to_string();
        let mut tx = self.pool.begin().await?;
        let (ctx, fingerprint) = context(&mut tx, &input.week_start).await?;
        let (mut doc, revision, mut generated, mut old) =
            document(&mut tx, &input.week_start, &ctx).await?;
        if replay(&mut tx, &input.action_id, &request).await? {
            let result = view(
                input.week_start,
                revision,
                generated,
                generated && old != fingerprint,
                doc,
                &ctx,
            )?;
            tx.commit().await?;
            return Ok(result);
        }
        if revision != input.expected_revision {
            return Err(AppError::conflict());
        }
        match &input.operation {
            ShoppingOperation::Refresh => {
                doc = merge(&doc, generate(&ctx)?);
                generated = true;
                old = fingerprint.clone();
            }
            ShoppingOperation::Manual {
                item_id,
                label,
                amount,
                unit_id,
                section_id,
            } => {
                validation::id(item_id)?;
                validation::text(label, 200, true)?;
                validation::optional_id(section_id)?;
                let adjustment = Override {
                    amount: amount.clone(),
                    unit_id: unit_id.clone(),
                };
                validate_override(&adjustment, &ctx)?;
                if section_id
                    .as_ref()
                    .is_some_and(|id| !ctx.sections.iter().any(|s| &s.id == id && !s.retired))
                {
                    return Err(AppError::validation("Choose an active shopping section."));
                }
                if doc.items.iter().any(|i| &i.id == item_id) {
                    return Err(AppError::conflict());
                }
                if doc.items.iter().filter(|i| i.key.is_none()).count() >= 500 {
                    return Err(AppError::validation(
                        "At most 500 manual shopping items per week.",
                    ));
                }
                doc.items.push(ShoppingItem {
                    id: item_id.clone(),
                    key: None,
                    ingredient_id: None,
                    label: label.trim().into(),
                    section_id: section_id.clone(),
                    numerator: None,
                    denominator: None,
                    unit_id: unit_id.clone(),
                    amount_text: amount.clone(),
                    approximate: false,
                    warnings: vec![],
                    rule_ids: vec![],
                    sources: vec![],
                    adjustment: Some(adjustment),
                    checked: false,
                    excluded: false,
                    orphaned: false,
                    legacy: None,
                });
            }
            operation => {
                let id = match operation {
                    ShoppingOperation::Adjust { item_id, .. }
                    | ShoppingOperation::Reset { item_id, .. }
                    | ShoppingOperation::Check { item_id, .. }
                    | ShoppingOperation::Exclude { item_id, .. } => item_id,
                    _ => unreachable!(),
                };
                // Generated IDs are backend-owned keys, not user-supplied UUIDs.
                validation::text(id, 500, true)?;
                let item = doc
                    .items
                    .iter_mut()
                    .find(|i| &i.id == id)
                    .ok_or_else(AppError::not_found)?;
                match operation {
                    ShoppingOperation::Adjust {
                        amount, unit_id, ..
                    } => {
                        let value = Override {
                            amount: amount.clone(),
                            unit_id: unit_id.clone(),
                        };
                        validate_override(&value, &ctx)?;
                        item.adjustment = Some(value);
                    }
                    ShoppingOperation::Reset { confirmed, .. } => {
                        super::catalog::confirmation(*confirmed)?;
                        if item.key.is_none() {
                            return Err(AppError::validation(
                                "Manual quantities can be edited, not reset.",
                            ));
                        }
                        item.adjustment = None;
                    }
                    ShoppingOperation::Check { checked, .. } => item.checked = *checked,
                    ShoppingOperation::Exclude { excluded, .. } => item.excluded = *excluded,
                    _ => unreachable!(),
                }
            }
        }
        let next = revision.checked_add(1).ok_or_else(AppError::conflict)?;
        let raw = serde_json::to_string(&doc).map_err(|_| AppError::storage())?;
        // generation_revision counts explicit refreshes; revision guards ALL edits.
        let refresh = matches!(input.operation, ShoppingOperation::Refresh);
        sqlx::query("INSERT INTO shopping_list(id,week_start,revision,generation_revision,generation_metadata_json,document_json) VALUES (?,?,?,?,?,?) ON CONFLICT(week_start) DO UPDATE SET revision=excluded.revision,generation_revision=shopping_list.generation_revision+?,generation_metadata_json=excluded.generation_metadata_json,document_json=excluded.document_json,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')")
            .bind(uuid::Uuid::new_v4().to_string()).bind(&input.week_start).bind(next).bind(i64::from(refresh)).bind(if generated {old.clone()} else {"{}".into()}).bind(raw).bind(i64::from(refresh)).execute(&mut *tx).await?;
        receipt(&mut tx, &input.action_id, &request).await?;
        let result = view(
            input.week_start,
            next,
            generated,
            generated && old != fingerprint,
            doc,
            &ctx,
        )?;
        tx.commit().await?;
        Ok(result)
    }
}
