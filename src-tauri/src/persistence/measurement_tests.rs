use super::*;
use crate::domain::measurements::*;
use tempfile::TempDir;
fn request(amount: &str, from: &str, to: &str, ingredient: Option<&str>) -> ConversionRequest {
    ConversionRequest {
        amount: amount.into(),
        from_unit: from.into(),
        to_unit: to.into(),
        ingredient_id: ingredient.map(str::to_string),
        purpose: Purpose::Quantity,
    }
}
async fn exact(
    db: &Database,
    amount: &str,
    from: &str,
    to: &str,
    ingredient: Option<&str>,
) -> String {
    match db
        .convert_measurement(request(amount, from, to, ingredient))
        .await
        .unwrap()
    {
        ConversionResult::Converted { exact, .. } => exact,
        result => panic!("{result:?}"),
    }
}
fn rule(ingredient: &str, from: &str, to: &str, amount: &str, approximate: bool) -> RuleInput {
    RuleInput {
        ingredient_id: ingredient.into(),
        from_unit_id: from.into(),
        from_amount: "1".into(),
        to_unit_id: to.into(),
        to_amount: amount.into(),
        approximate,
        notes: "Medium size; household estimate".into(),
    }
}
#[tokio::test]
async fn registry_mass_volume_roundtrips_and_design_vectors() {
    let dir = TempDir::new().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    assert_eq!(exact(&db, "1", "lb", "g", None).await, "45359237/100000");
    assert_eq!(exact(&db, "1", "oz", "mg", None).await, "45359237/1600");
    assert_eq!(exact(&db, "1", "kg", "mg", None).await, "1000000");
    assert_eq!(exact(&db, "1", "cup-metric", "ml", None).await, "250");
    assert_eq!(
        exact(&db, "1", "cup-us", "ml", None).await,
        "473176473/2000000"
    );
    assert_eq!(
        exact(&db, "1", "cup-imperial", "ml", None).await,
        "454609/1600"
    );
    // Every compatible pair roundtrips, including the less common fluid variants.
    let catalog = db.catalog().await.unwrap();
    let prefs = db.measurement_preferences().await.unwrap();
    for from in &catalog.units {
        for to in &catalog.units {
            if from.dimension == to.dimension
                && matches!(from.dimension.as_str(), "mass" | "volume")
            {
                let result = convert(
                    request("1/3", &from.id, &to.id, None),
                    &catalog.units,
                    &[],
                    &[],
                    &prefs,
                )
                .unwrap();
                if let ConversionResult::Converted {
                    numerator,
                    denominator,
                    ..
                } = result
                {
                    // Roundtrip exact values internally: outputs may exceed input precision limits.
                    let source = rational::Rational::new(
                        numerator.parse().unwrap(),
                        denominator.parse().unwrap(),
                    )
                    .unwrap();
                    let from_factor = rational::Rational::new(
                        from.factor_numerator.unwrap().into(),
                        from.factor_denominator.unwrap().into(),
                    )
                    .unwrap();
                    let to_factor = rational::Rational::new(
                        to.factor_numerator.unwrap().into(),
                        to.factor_denominator.unwrap().into(),
                    )
                    .unwrap();
                    assert_eq!(
                        source
                            .checked_mul(to_factor)
                            .unwrap()
                            .checked_div(from_factor)
                            .unwrap(),
                        rational::Rational::new(1, 3).unwrap()
                    );
                } else {
                    panic!("Compatible unit pair failed");
                }
            }
        }
    }
    let g = rational::parse(&exact(&db, "1", "lb", "g", None).await)
        .unwrap()
        .unwrap();
    assert_eq!(
        g.checked_add(rational::parse("500").unwrap().unwrap())
            .unwrap()
            .fraction(),
        "95359237/100000"
    );
    assert_eq!(
        rational::parse(&exact(&db, "1", "cup-metric", "ml", None).await)
            .unwrap()
            .unwrap()
            .checked_add(rational::parse("250").unwrap().unwrap())
            .unwrap()
            .fraction(),
        "500"
    );
}
#[tokio::test]
async fn ambiguity_unknown_incompatible_and_temperature_isolation() {
    let dir = TempDir::new().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    for (amount, from, to) in [
        ("1", "cup", "ml"),
        ("1", "tsp", "ml"),
        ("1", "fl oz", "ml"),
        ("1", "can", "g"),
        ("2", "piece", "g"),
        ("1", "cup-us", "g"),
        ("to taste", "to-taste", "g"),
        ("1", "pinch", "g"),
        ("1", "celsius", "g"),
        ("1", "celsius", "fahrenheit"),
    ] {
        assert!(matches!(
            db.convert_measurement(request(amount, from, to, None))
                .await
                .unwrap(),
            ConversionResult::CannotConvert { .. }
        ));
    }
    let mut prefs = db.measurement_preferences().await.unwrap();
    prefs.cup_convention = Some(CupConvention::Metric);
    db.save_measurement_preferences(prefs).await.unwrap();
    // Choosing a review default does not automatically rewrite ambiguous source units.
    if let ConversionResult::CannotConvert { candidates, .. } = db
        .convert_measurement(request("1", "cup", "ml", None))
        .await
        .unwrap()
    {
        assert_eq!(
            candidates
                .iter()
                .filter(|id| id.starts_with("cup-"))
                .count(),
            3
        );
    } else {
        panic!("Ambiguous cup converted");
    }
    for (amount, from, to, expected) in [
        ("0", "celsius", "fahrenheit", "32"),
        ("212", "fahrenheit", "celsius", "100"),
        ("-40", "celsius", "fahrenheit", "-40"),
        ("350", "fahrenheit", "celsius", "530/3"),
    ] {
        let mut input = request(amount, from, to, None);
        input.purpose = Purpose::Temperature;
        if let ConversionResult::Converted { exact, .. } =
            db.convert_measurement(input).await.unwrap()
        {
            assert_eq!(exact, expected);
        } else {
            panic!("Temperature display failed");
        }
    }
    assert!(db
        .convert_measurement(request("-1", "g", "kg", None))
        .await
        .is_err());
}
#[tokio::test]
async fn ingredient_rules_scope_approximations_reverse_ambiguity_and_retirement() {
    let dir = TempDir::new().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let onion = db
        .save_conversion_rule(None, rule("ingredient-onion", "piece", "g", "150", true))
        .await
        .unwrap();
    let result = db
        .convert_measurement(request("2", "piece", "kg", Some("ingredient-onion")))
        .await
        .unwrap();
    if let ConversionResult::Converted {
        exact,
        approximate,
        rule_ids,
        original,
        ..
    } = result
    {
        assert_eq!(exact, "3/10");
        assert!(approximate);
        assert_eq!(rule_ids, vec![onion.id.clone()]);
        assert_eq!(original.amount, "2");
    } else {
        panic!("Missing rule");
    }
    assert_eq!(
        exact(&db, "300", "g", "piece", Some("ingredient-onion")).await,
        "2"
    );
    assert!(matches!(
        db.convert_measurement(request("2", "piece", "g", Some("ingredient-apple")))
            .await
            .unwrap(),
        ConversionResult::CannotConvert { .. }
    ));
    db.save_conversion_rule(None, rule("ingredient-flour", "cup-us", "g", "120", true))
        .await
        .unwrap();
    assert_eq!(
        exact(&db, "1/2", "cup-us", "g", Some("ingredient-flour")).await,
        "60"
    );
    let package = db
        .save_conversion_rule(None, rule("ingredient-tomato", "can", "g", "400", false))
        .await
        .unwrap();
    assert_eq!(
        exact(&db, "1", "can", "kg", Some("ingredient-tomato")).await,
        "2/5"
    );
    assert!(db
        .save_conversion_rule(None, rule("ingredient-onion", "g", "piece", "1", false))
        .await
        .is_err());
    db.save_conversion_rule(None, rule("ingredient-onion", "piece", "kg", "1/5", true))
        .await
        .unwrap();
    assert!(matches!(
        db.convert_measurement(request("1", "piece", "g", Some("ingredient-onion")))
            .await
            .unwrap(),
        ConversionResult::CannotConvert { .. }
    ));
    assert!(db.retire_conversion_rule(&package.id, false).await.is_err());
    db.retire_conversion_rule(&package.id, true).await.unwrap();
    assert!(db
        .conversion_rules("ingredient-tomato")
        .await
        .unwrap()
        .is_empty());
    assert!(matches!(
        db.convert_measurement(request("1", "can", "g", Some("ingredient-tomato")))
            .await
            .unwrap(),
        ConversionResult::CannotConvert { .. }
    ));
    for input in [
        rule("ingredient-onion", "g", "kg", "1", false),
        rule("ingredient-onion", "piece", "g", "0", false),
        rule("ingredient-onion", "piece", "celsius", "1", false),
        rule("missing", "piece", "g", "1", false),
    ] {
        assert!(db.save_conversion_rule(None, input).await.is_err());
    }
}
#[tokio::test]
async fn rules_preferences_and_identity_survive_restart_and_rejected_edits() {
    let dir = TempDir::new().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let mut prefs = db.measurement_preferences().await.unwrap();
    assert_eq!(prefs.mode, Mode::Mixed);
    assert_eq!(prefs.cup_convention, None);
    prefs.mode = Mode::Us;
    prefs.fractions = false;
    prefs.decimal_places = 3;
    prefs.aggregate_compatible = false;
    db.save_measurement_preferences(prefs.clone())
        .await
        .unwrap();
    let mut bad = prefs.clone();
    bad.decimal_places = 7;
    assert!(db.save_measurement_preferences(bad).await.is_err());
    let mut input = rule("ingredient-onion", "piece", "g", "1 1/2", true);
    input.from_amount = "½".into();
    let stored = db.save_conversion_rule(None, input).await.unwrap();
    assert_eq!(stored.original_from_text, "½");
    assert_eq!(stored.from_denominator, 2);
    let candidates = db
        .ingredient_candidates(vec!["  ONIONS  ".into()], None)
        .await
        .unwrap();
    assert_eq!(candidates[0].id, "ingredient-onion");
    assert!(db
        .ingredient_candidates(vec!["onions".into()], Some("ingredient-onion".into()))
        .await
        .unwrap()
        .is_empty());
    db.retire_ingredient("ingredient-garlic", true)
        .await
        .unwrap();
    assert!(
        db.ingredient_candidates(vec!["Garlic".into()], None)
            .await
            .unwrap()[0]
            .retired
    );
    db.close().await;
    let db = Database::open(dir.path()).await.unwrap();
    assert_eq!(db.measurement_preferences().await.unwrap(), prefs);
    assert_eq!(
        db.conversion_rules("ingredient-onion").await.unwrap()[0].original_from_text,
        "½"
    );
    if let ConversionResult::Converted {
        display,
        exact,
        rounded,
        ..
    } = db
        .convert_measurement(request("1/3", "g", "g", None))
        .await
        .unwrap()
    {
        assert_eq!(display, "0.333");
        assert_eq!(exact, "1/3");
        assert!(rounded);
    } else {
        panic!("Decimal display failed");
    }
}

#[tokio::test]
async fn rule_edits_are_atomic_stable_and_validated() {
    let dir = TempDir::new().unwrap();
    let db = Database::open(dir.path()).await.unwrap();
    let first = db
        .save_conversion_rule(None, rule("ingredient-onion", "piece", "g", "150", true))
        .await
        .unwrap();
    let mut update = rule("ingredient-onion", "piece", "g", "160", true);
    update.notes = "Large onion".into();
    let second = db
        .save_conversion_rule(Some(first.id.clone()), update)
        .await
        .unwrap();
    assert_eq!(second.id, first.id);
    assert_eq!(second.to_numerator, 160);
    assert_eq!(second.notes, "Large onion");
    assert_eq!(
        exact(&db, "2", "piece", "g", Some("ingredient-onion")).await,
        "320"
    );
    for (id, input) in [
        (
            Some(first.id.clone()),
            rule("ingredient-onion", "piece", "g", "0", true),
        ),
        (
            Some("missing-rule".into()),
            rule("ingredient-onion", "piece", "g", "150", true),
        ),
        (
            Some(first.id.clone()),
            rule("ingredient-apple", "piece", "g", "150", true),
        ),
    ] {
        assert!(db.save_conversion_rule(id, input).await.is_err());
    }
    assert_eq!(
        db.conversion_rules("ingredient-onion").await.unwrap()[0].to_numerator,
        160
    );
    db.retire_conversion_rule(&first.id, true).await.unwrap();
    assert!(db
        .save_conversion_rule(
            Some(first.id),
            rule("ingredient-onion", "piece", "g", "150", true)
        )
        .await
        .is_err());
    let mut onion = db.ingredient("ingredient-onion").await.unwrap().data;
    onion.preferred_unit_id = Some("celsius".into());
    assert!(db
        .update_ingredient("ingredient-onion", onion)
        .await
        .is_err());
    assert_ne!(
        db.ingredient("ingredient-onion")
            .await
            .unwrap()
            .data
            .preferred_unit_id,
        Some("celsius".into())
    );
}
