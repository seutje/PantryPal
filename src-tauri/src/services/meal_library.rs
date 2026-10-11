use crate::{
    domain::{
        meal_library::*,
        meals::{MealIngredient, MealInput},
        measurements::rational::{self, Rational},
        validation::{self, Amount},
    },
    error::{AppError, AppResult},
    persistence::Database,
};
use chrono::NaiveDate;
fn quantity(raw: &str, positive: bool) -> AppResult<Option<Amount>> {
    let value = rational::parse(raw)?.map(|r| Amount {
        numerator: r.n as i64,
        denominator: r.d as i64,
    });
    if let Some(amount) = &value {
        amount.validate(positive)?;
    } else if positive {
        return Err(AppError::validation(
            "Base servings must be known and greater than zero.",
        ));
    }
    Ok(value)
}
fn pair(amount: &Amount) -> String {
    if amount.denominator == 1 {
        amount.numerator.to_string()
    } else {
        format!("{}/{}", amount.numerator, amount.denominator)
    }
}
fn date(value: &Option<String>) -> AppResult<()> {
    if let Some(value) = value {
        if !NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .is_ok_and(|d| d.format("%Y-%m-%d").to_string() == *value)
        {
            return Err(AppError::validation("Choose a valid cooked date."));
        }
    }
    Ok(())
}
impl Database {
    pub async fn save_meal_draft(
        &self,
        id: Option<String>,
        expected_revision: Option<i64>,
        draft: MealDraft,
    ) -> AppResult<crate::domain::meals::Meal> {
        validation::optional_id(&id)?;
        let old = if let Some(id) = &id {
            Some(self.meal(id).await?)
        } else {
            None
        };
        if id.is_some() != expected_revision.is_some() {
            return Err(AppError::validation(
                "Meal edits require the saved revision.",
            ));
        }
        let input = draft_input(draft, old.as_ref())?;
        if let Some(id) = id {
            self.update_meal(&id, expected_revision.unwrap(), input)
                .await
        } else {
            self.create_meal(input).await
        }
    }
    pub async fn search_meals(&self, input: MealSearch) -> AppResult<MealLibrary> {
        validation::text(&input.text, 200, false)?;
        validation::text(&input.tag, 100, false)?;
        validation::optional_id(&input.ingredient_id)?;
        date(&input.cooked_from)?;
        date(&input.cooked_to)?;
        if matches!((&input.cooked_from,&input.cooked_to),(Some(a),Some(b)) if a>b) {
            return Err(AppError::validation("The cooked date range is reversed."));
        }
        let fields = self.all_fields().await?;
        if let Some(filter) = &input.field {
            let field = fields
                .iter()
                .find(|f| f.id == filter.field_id)
                .ok_or_else(AppError::not_found)?;
            field.data.validate_value(&filter.value)?;
            if matches!(filter.operator, FieldOperator::Equals) && filter.value.is_none() {
                return Err(AppError::validation(
                    "Choose a field value, or filter by unknown.",
                ));
            }
        }
        let all = self.list_meals().await?;
        let mut catalog = self.catalog().await?;
        for meal in &all {
            for line in &meal.data.ingredients {
                if let Some(id) = &line.ingredient_id {
                    if !catalog.ingredients.iter().any(|i| i.id == *id) {
                        catalog.ingredients.push(self.ingredient(id).await?);
                    }
                }
            }
        }
        let prefs = self.measurement_preferences().await?;
        let display = |amount: &Amount| -> AppResult<String> {
            let r = Rational::new(amount.numerator.into(), amount.denominator.into())?;
            if prefs.fractions {
                Ok(r.display_fraction())
            } else {
                let (text, rounded) = r.decimal(prefs.decimal_places)?;
                Ok(if rounded {
                    format!("{text} (rounded; exact {})", r.fraction())
                } else {
                    text
                })
            }
        };
        let text = validation::normalized(&input.text);
        let mut meals = vec![];
        for meal in all {
            let last_cooked: Option<String> = self.last_cooked(&meal.id).await?;
            if matches!(input.cooked, CookedFilter::Never) && last_cooked.is_some()
                || matches!(input.cooked, CookedFilter::Cooked) && last_cooked.is_none()
            {
                continue;
            }
            if input
                .cooked_from
                .as_ref()
                .is_some_and(|from| last_cooked.as_ref().is_none_or(|d| d < from))
                || input
                    .cooked_to
                    .as_ref()
                    .is_some_and(|to| last_cooked.as_ref().is_none_or(|d| d > to))
            {
                continue;
            }
            if input.ingredient_id.as_ref().is_some_and(|id| {
                !meal
                    .data
                    .ingredients
                    .iter()
                    .any(|l| l.ingredient_id.as_ref() == Some(id))
            }) {
                continue;
            }
            if !input.tag.trim().is_empty()
                && !meal
                    .data
                    .tags
                    .iter()
                    .any(|t| validation::normalized(t) == validation::normalized(&input.tag))
            {
                continue;
            }
            if let Some(filter) = &input.field {
                let value = meal
                    .data
                    .custom_fields
                    .iter()
                    .find(|v| v.field_id == filter.field_id)
                    .and_then(|v| v.value.as_ref());
                let matched = match filter.operator {
                    FieldOperator::Unknown => value.is_none(),
                    FieldOperator::Known => value.is_some(),
                    FieldOperator::Equals => value == filter.value.as_ref(),
                };
                if !matched {
                    continue;
                }
            }
            let mut haystack = vec![meal.data.title.clone(), meal.data.description.clone()];
            haystack.extend(meal.data.tags.clone());
            for line in &meal.data.ingredients {
                haystack.push(line.raw_text.clone());
                if let Some(ingredient) = catalog
                    .ingredients
                    .iter()
                    .find(|i| Some(&i.id) == line.ingredient_id.as_ref())
                {
                    haystack.push(ingredient.data.name.clone());
                    haystack.extend(ingredient.data.aliases.clone());
                }
            }
            for value in &meal.data.custom_fields {
                if let Some(crate::domain::fields::FieldValue::Select(option_id)) = &value.value {
                    if let Some(option) = fields
                        .iter()
                        .find(|field| field.id == value.field_id)
                        .and_then(|field| {
                            field
                                .data
                                .options
                                .iter()
                                .find(|option| option.id == *option_id)
                        })
                    {
                        haystack.push(option.label.clone());
                    }
                }
                if let Some(value) = &value.value {
                    haystack.push(serde_json::to_string(value).map_err(|_| AppError::storage())?);
                }
            }
            if !text.is_empty()
                && !haystack
                    .iter()
                    .any(|s| validation::normalized(s).contains(&text))
            {
                continue;
            }
            let servings_display = display(&meal.data.base_servings)?;
            let quantity_displays = meal
                .data
                .ingredients
                .iter()
                .map(|l| l.quantity.as_ref().map(display).transpose())
                .collect::<AppResult<_>>()?;
            meals.push(MealRecord {
                meal,
                last_cooked,
                servings_display,
                quantity_displays,
            });
        }
        Ok(MealLibrary {
            meals,
            catalog,
            fields,
        })
    }
}

pub(crate) fn draft_input(
    draft: MealDraft,
    old: Option<&crate::domain::meals::Meal>,
) -> AppResult<MealInput> {
    let base_servings = if let Some(old) = old {
        if draft.servings
            == old
                .data
                .base_servings_text
                .clone()
                .unwrap_or_else(|| pair(&old.data.base_servings))
        {
            old.data.base_servings.clone()
        } else {
            quantity(&draft.servings, true)?.unwrap()
        }
    } else {
        quantity(&draft.servings, true)?.unwrap()
    };
    if draft.ingredients.len() > 300 {
        return Err(AppError::validation("Too many ingredient lines."));
    }
    let mut ingredients = vec![];
    for line in draft.ingredients {
        let previous = old
            .as_ref()
            .and_then(|m| m.data.ingredients.iter().find(|l| l.id == line.id));
        let amount = if let Some(previous) = previous {
            if line.amount
                == previous
                    .amount_text
                    .clone()
                    .unwrap_or_else(|| previous.quantity.as_ref().map(pair).unwrap_or_default())
            {
                previous.quantity.clone()
            } else {
                quantity(&line.amount, false)?
            }
        } else {
            quantity(&line.amount, false)?
        };
        ingredients.push(MealIngredient {
            id: line.id,
            ingredient_id: line.ingredient_id,
            quantity: amount,
            amount_text: Some(line.amount),
            unit_id: line.unit_id,
            raw_text: line.raw_text,
            source_convention: line.source_convention,
            note: line.note,
            optional: line.optional,
        });
    }
    Ok(MealInput {
        title: draft.title,
        description: draft.description,
        base_servings,
        base_servings_text: Some(draft.servings),
        prep_minutes: draft.prep_minutes,
        cook_minutes: draft.cook_minutes,
        instructions: draft.instructions,
        notes: draft.notes,
        source_url: draft.source_url,
        source_site: draft.source_site,
        tags: draft.tags,
        ingredients,
        custom_fields: draft.custom_fields,
    })
}
