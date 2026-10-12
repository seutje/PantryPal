use crate::{
    domain::{
        meal_library::{CookedFilter, MealRecord, MealSearch},
        measurements::rational::Rational,
        planning::*,
    },
    error::AppResult,
    persistence::Database,
};
use chrono::Days;
impl Database {
    pub async fn get_week(&self, date: &str) -> AppResult<WeekView> {
        let days = week_dates(date)?;
        let monday = local_date(&days[0])?;
        let previous_week = monday
            .checked_sub_days(Days::new(7))
            .map(|d| d.format("%Y-%m-%d").to_string())
            .filter(|d| validate_week(d).is_ok());
        let next_week = monday
            .checked_add_days(Days::new(7))
            .map(|d| d.format("%Y-%m-%d").to_string())
            .filter(|d| validate_week(d).is_ok());
        let (revision, mut entries) = self.week_entries(&days[0]).await?;
        let mut library = self
            .search_meals(MealSearch {
                text: String::new(),
                ingredient_id: None,
                tag: String::new(),
                field: None,
                cooked: CookedFilter::Any,
                cooked_from: None,
                cooked_to: None,
            })
            .await?;
        let prefs = self.measurement_preferences().await?;
        let display = |amount: &crate::domain::validation::Amount| -> AppResult<String> {
            let q = Rational::new(amount.numerator.into(), amount.denominator.into())?;
            if prefs.fractions {
                Ok(q.display_fraction())
            } else {
                let (text, rounded) = q.decimal(prefs.decimal_places)?;
                Ok(if rounded {
                    format!("{text} (rounded; exact {})", q.fraction())
                } else {
                    text
                })
            }
        };
        for entry in &entries {
            if !library.meals.iter().any(|m| m.meal.id == entry.meal_id) {
                let meal = self.meal(&entry.meal_id).await?;
                for line in &meal.data.ingredients {
                    if let Some(id) = &line.ingredient_id {
                        if !library.catalog.ingredients.iter().any(|i| i.id == *id) {
                            library.catalog.ingredients.push(self.ingredient(id).await?);
                        }
                    }
                }
                let last_cooked = self.last_cooked(&meal.id).await?;
                let servings_display = display(&meal.data.base_servings)?;
                let quantity_displays = meal
                    .data
                    .ingredients
                    .iter()
                    .map(|l| l.quantity.as_ref().map(display).transpose())
                    .collect::<AppResult<_>>()?;
                library.meals.push(MealRecord {
                    meal,
                    last_cooked,
                    servings_display,
                    quantity_displays,
                });
            }
        }
        for entry in &mut entries {
            let record = library
                .meals
                .iter()
                .find(|m| m.meal.id == entry.meal_id)
                .ok_or_else(crate::error::AppError::storage)?;
            let scale = Rational::new(
                entry.servings.numerator.into(),
                entry.servings.denominator.into(),
            )?
            .checked_div(Rational::new(
                record.meal.data.base_servings.numerator.into(),
                record.meal.data.base_servings.denominator.into(),
            )?)?;
            entry.quantity_displays = record
                .meal
                .data
                .ingredients
                .iter()
                .map(|line| {
                    line.quantity
                        .as_ref()
                        .map(|amount| {
                            let original =
                                Rational::new(amount.numerator.into(), amount.denominator.into())?;
                            let temperature = library.catalog.units.iter().any(|u| {
                                Some(&u.id) == line.unit_id.as_ref() && u.dimension == "temperature"
                            });
                            let q = if temperature {
                                original
                            } else {
                                original.checked_mul(scale)?
                            };
                            if prefs.fractions {
                                Ok(q.display_fraction())
                            } else {
                                let (text, rounded) = q.decimal(prefs.decimal_places)?;
                                Ok(if rounded {
                                    format!("{text} (rounded; exact {})", q.fraction())
                                } else {
                                    text
                                })
                            }
                        })
                        .transpose()
                })
                .collect::<AppResult<_>>()?;
        }
        Ok(WeekView {
            week_start: days[0].clone(),
            week_end: days[6].clone(),
            previous_week,
            next_week,
            days,
            revision,
            entries,
            library,
        })
    }
    pub async fn apply_plan(&self, input: PlanMutation) -> AppResult<WeekView> {
        self.mutate_plan(&input).await?;
        self.get_week(&input.week_start).await
    }
}
