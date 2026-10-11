use super::*;
#[test]
fn fraction_decimal_unicode_and_unknown_parsing() {
    for (text, n, d) in [
        ("1/2", 1, 2),
        ("½", 1, 2),
        ("1 1/2", 3, 2),
        ("1½", 3, 2),
        ("⅓", 1, 3),
        ("0.125", 1, 8),
        (".5", 1, 2),
        ("-1 1/2", -3, 2),
        ("0", 0, 1),
    ] {
        assert_eq!(
            parse(text).unwrap(),
            Some(Rational::new(n, d).unwrap()),
            "{text}"
        );
    }
    for text in ["", "to taste", "unknown", "as needed"] {
        assert_eq!(parse(text).unwrap(), None);
    }
    for text in [
        "1/0",
        "1e3",
        "1-2",
        "NaN",
        "1 3/2",
        "1/2/3",
        "-",
        "1.1234567890",
        "1000000001",
        "1.2.3",
        "½½",
    ] {
        assert!(parse(text).is_err(), "{text}");
    }
}
#[test]
fn arithmetic_roundtrips_scaling_and_limits() {
    let third = parse("1/3").unwrap().unwrap();
    let quantity = parse("2.5").unwrap().unwrap();
    let planned = parse("1.5").unwrap().unwrap();
    let base = parse("4").unwrap().unwrap();
    assert_eq!(
        quantity
            .checked_mul(planned)
            .unwrap()
            .checked_div(base)
            .unwrap()
            .fraction(),
        "15/16"
    );
    assert_eq!(
        parse("0.000000001").unwrap().unwrap(),
        Rational::new(1, 1_000_000_000).unwrap()
    );
    assert!(third.decimal(255).is_err());
    assert_eq!(third.display_fraction(), "⅓");
    assert_eq!(parse("-1 1/2").unwrap().unwrap().display_fraction(), "-1½");
    assert_eq!(parse("12/7").unwrap().unwrap().display_fraction(), "1 5/7");
    assert_eq!(
        third
            .checked_add(third)
            .unwrap()
            .checked_add(third)
            .unwrap(),
        Rational::new(1, 1).unwrap()
    );
    assert_eq!(
        third.checked_mul(Rational::new(3, 2).unwrap()).unwrap(),
        Rational::new(1, 2).unwrap()
    );
    assert_eq!(
        third
            .checked_mul(Rational::new(9, 7).unwrap())
            .unwrap()
            .checked_div(Rational::new(9, 7).unwrap())
            .unwrap(),
        third
    );
    assert_eq!(third.decimal(3).unwrap(), ("0.333".into(), true));
    assert_eq!(
        parse("-1.25").unwrap().unwrap().decimal(1).unwrap(),
        ("-1.3".into(), true)
    );
    assert_eq!(
        parse("0.0001").unwrap().unwrap().decimal(3).unwrap(),
        ("0".into(), true)
    );
    assert_eq!(
        parse("2.5").unwrap().unwrap().decimal(0).unwrap(),
        ("3".into(), true)
    );
    assert!(Rational::new(i128::MAX, 1)
        .unwrap()
        .checked_mul(Rational::new(2, 1).unwrap())
        .is_err());
    assert!(third.checked_div(Rational::new(0, 1).unwrap()).is_err());
    // Cross reduction avoids overflow of a large but reducible product.
    assert_eq!(
        Rational::new(i128::MAX, 2)
            .unwrap()
            .checked_mul(Rational::new(2, i128::MAX).unwrap())
            .unwrap(),
        Rational::new(1, 1).unwrap()
    );
}
