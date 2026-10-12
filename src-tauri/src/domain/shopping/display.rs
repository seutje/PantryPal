//! Shopping presentation only: exact stored totals and overrides are untouched.
use super::{Context, Rational};
use crate::{
    domain::catalog::Unit,
    error::{AppError, AppResult},
};

fn factor(unit: &Unit) -> AppResult<Rational> {
    let n = unit.factor_numerator.ok_or_else(AppError::storage)?;
    let d = unit.factor_denominator.ok_or_else(AppError::storage)?;
    if n <= 0 {
        return Err(AppError::storage());
    }
    Rational::new(n.into(), d.into())
}
fn convert(q: Rational, from: &Unit, to: &Unit) -> AppResult<Rational> {
    if from.dimension != to.dimension {
        return Err(AppError::storage());
    }
    q.checked_mul(factor(from)?)?.checked_div(factor(to)?)
}
fn ceil(q: Rational, scale: i128) -> AppResult<Rational> {
    if q.n < 0 {
        return Err(AppError::storage());
    }
    let scaled = q.checked_mul(Rational::new(scale, 1)?)?;
    let ticks = (scaled.n / scaled.d)
        .checked_add(i128::from(scaled.n % scaled.d != 0))
        .ok_or_else(AppError::storage)?;
    Rational::new(ticks, scale)
}
fn amount(q: Rational, unit: Option<&Unit>, ctx: &Context) -> AppResult<(String, bool)> {
    let whole =
        unit.is_some_and(|u| matches!(u.id.as_str(), "ml" | "g" | "mg") || u.dimension == "count");
    let simple = matches!(q.d, 1 | 2 | 3 | 4 | 5 | 6 | 8 | 12 | 16);
    let fractions = ctx.prefs.fractions
        && !whole
        && unit.is_none_or(|u| u.id != "l" && (u.id != "kg" || simple));
    let rounded = if whole {
        ceil(q, 1)?
    } else if fractions && simple {
        q
    } else if fractions {
        ceil(q, 8)?
    } else {
        ceil(q, 10_i128.pow(u32::from(ctx.prefs.decimal_places.min(2))))?
    };
    let text = if fractions {
        rounded.display_fraction()
    } else {
        rounded
            .decimal(if whole {
                0
            } else {
                ctx.prefs.decimal_places.min(2)
            })?
            .0
    };
    Ok((text, q != rounded))
}

fn measurement(q: Rational, unit: Option<&Unit>, ctx: &Context) -> AppResult<(String, bool)> {
    let (text, rounded) = amount(q, unit, ctx)?;
    Ok((
        format!(
            "{text} {}",
            unit.map(|u| u.symbol.as_str()).unwrap_or_default()
        )
        .trim()
        .into(),
        rounded,
    ))
}
pub(super) fn format(
    q: Rational,
    source: Option<&Unit>,
    ctx: &Context,
) -> AppResult<(String, bool, Vec<String>)> {
    let mut primary = source;
    let mut primary_q = q;
    if let Some(from) = source {
        let larger = match from.id.as_str() {
            "ml" => Some("l"),
            "g" => Some("kg"),
            _ => None,
        };
        if let Some(to) = larger.and_then(|id| ctx.units.iter().find(|u| u.id == id)) {
            let converted = convert(q, from, to)?;
            if converted.n >= converted.d {
                primary = Some(to);
                primary_q = converted;
            }
        }
    }
    let (display, rounded) = measurement(primary_q, primary, ctx)?;
    let mut alternatives = vec![];
    if let Some(from) = source.filter(|u| {
        matches!(u.dimension.as_str(), "volume" | "mass") && u.factor_numerator.is_some()
    }) {
        let targets: &[&str] = if from.dimension == "volume" {
            &[
                "ml",
                "l",
                "cup-us",
                "cup-metric",
                "cup-imperial",
                "floz-us",
                "floz-imperial",
            ]
        } else {
            &["g", "kg", "oz", "lb"]
        };
        for id in targets {
            let Some(to) = ctx.units.iter().find(|u| &u.id == id) else {
                continue;
            };
            if primary.is_some_and(|u| u.id == to.id) {
                continue;
            }
            // Do not provide a second enormous metric number beside kg/L.
            if primary.is_some_and(|u| matches!(u.id.as_str(), "kg" | "l"))
                && matches!(to.id.as_str(), "g" | "ml")
            {
                continue;
            }
            let value = convert(q, from, to)?;
            alternatives.push(measurement(value, Some(to), ctx)?.0);
        }
    }
    Ok((display, rounded, alternatives))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::measurements::rational::parse, persistence::Database};
    async fn context() -> Context {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path()).await.unwrap();
        let catalog = db.catalog().await.unwrap();
        Context {
            entries: vec![],
            meals: vec![],
            ingredients: catalog.ingredients,
            units: catalog.units,
            sections: catalog.sections,
            rules: vec![],
            prefs: db.measurement_preferences().await.unwrap(),
        }
    }
    #[tokio::test]
    async fn olive_oil_regression_has_readable_regional_alternatives() {
        let ctx = context().await;
        let q = Rational::new(473176473, 8000000).unwrap();
        let ml = ctx.units.iter().find(|u| u.id == "ml").unwrap();
        let (shown, rounded, alternatives) = format(q, Some(ml), &ctx).unwrap();
        assert_eq!(shown, "60 mL");
        assert!(rounded);
        assert!(alternatives.contains(&"0.06 L".into()));
        assert!(alternatives.contains(&"¼ US cup".into()));
        assert!(alternatives.contains(&"¼ metric cup".into()));
        assert!(alternatives.contains(&"¼ Imperial cup".into()));
        assert!(alternatives.contains(&"2 US fl oz".into()));
        assert!(alternatives.contains(&"2⅛ Imperial fl oz".into()));
    }
    #[tokio::test]
    async fn rounding_never_understates_quantities_across_preferences_and_units() {
        let mut ctx = context().await;
        for fractions in [true, false] {
            for places in [0, 2, 6] {
                ctx.prefs.fractions = fractions;
                ctx.prefs.decimal_places = places;
                for unit in &ctx.units {
                    if unit.dimension == "temperature" {
                        continue;
                    }
                    for (n, d) in [
                        (0, 1),
                        (1, 3),
                        (1, 1000000000),
                        (473176473, 8000000),
                        (150001, 100),
                        (1000000000000000000000, 1),
                    ] {
                        let q = Rational::new(n, d).unwrap();
                        let (shown, _) = amount(q, Some(unit), &ctx).unwrap();
                        let rounded = if n > 1_000_000_000 {
                            Rational::new(shown.parse::<i128>().unwrap(), 1).unwrap()
                        } else {
                            parse(&shown).unwrap().unwrap()
                        };
                        assert!(
                            rounded.checked_add(q.negated()).unwrap().n >= 0,
                            "{shown} {}",
                            unit.id
                        );
                        assert!(rounded.d <= 100 || (fractions && rounded.d <= 16));
                    }
                }
            }
        }
    }
    #[tokio::test]
    async fn scale_and_dimension_boundaries_remain_explicit() {
        let ctx = context().await;
        let ml = ctx.units.iter().find(|u| u.id == "ml").unwrap();
        assert_eq!(
            format(Rational::new(1500001, 1000).unwrap(), Some(ml), &ctx)
                .unwrap()
                .0,
            "1.51 L"
        );
        let g = ctx.units.iter().find(|u| u.id == "g").unwrap();
        let (shown, _, alternatives) =
            format(Rational::new(1500001, 1000).unwrap(), Some(g), &ctx).unwrap();
        assert_eq!(shown, "1.51 kg");
        assert!(alternatives
            .iter()
            .all(|s| !s.contains("cup") && !s.contains("fl oz")));
        assert!(convert(Rational::new(1, 1).unwrap(), ml, g).is_err());
        let piece = ctx.units.iter().find(|u| u.id == "piece").unwrap();
        let (shown, rounded, alternatives) =
            format(Rational::new(3, 2).unwrap(), Some(piece), &ctx).unwrap();
        assert_eq!(shown, "2 pcs");
        assert!(rounded);
        assert!(alternatives.is_empty());
        assert!(ceil(Rational::new(i128::MAX, 1).unwrap(), 100).is_err());
    }
}
