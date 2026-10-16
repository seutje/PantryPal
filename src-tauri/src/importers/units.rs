use crate::domain::measurements::Mode;

// Longest whole-token match prevents `fl oz` becoming `oz`, and avoids
// interpreting ingredient names such as ground beef as a gram abbreviation.
pub(super) fn prefix(raw: &str, mode: &Mode) -> Option<(String, Option<String>, usize)> {
    let words: Vec<_> = raw.split_whitespace().collect();
    for count in (1..=words.len().min(4)).rev() {
        let text = words[..count].join(" ").to_lowercase();
        let text = text.trim_end_matches('.');
        let (region, text) = ["imperial", "metric", "us"]
            .into_iter()
            .find_map(|region| {
                text.strip_prefix(&format!("{region} "))
                    .map(|rest| (Some(region), rest))
            })
            .unwrap_or((None, text));
        let family = match text {
            "cup" | "cups" => Some("cup"),
            "tsp" | "tsps" | "teaspoon" | "teaspoons" => Some("tsp"),
            "tbsp" | "tbsps" | "tbs" | "tablespoon" | "tablespoons" => Some("tbsp"),
            "fl oz" | "fl. oz" | "fluid ounce" | "fluid ounces" => Some("floz"),
            "pt" | "pts" | "pint" | "pints" => Some("pint"),
            "qt" | "qts" | "quart" | "quarts" => Some("quart"),
            "gal" | "gals" | "gallon" | "gallons" => Some("gallon"),
            _ => None,
        };
        let resolved = if let Some(family) = family {
            let region = region.unwrap_or(
                if *mode == Mode::Metric && matches!(family, "cup" | "tsp" | "tbsp") {
                    "metric"
                } else {
                    "us"
                },
            );
            if region == "metric" && !matches!(family, "cup" | "tsp" | "tbsp") {
                continue;
            }
            Some((format!("{family}-{region}"), Some(region.to_owned())))
        } else if region.is_none() {
            let id = match text {
                "mg" | "milligram" | "milligrams" => "mg",
                "g" | "gram" | "grams" => "g",
                "kg" | "kilogram" | "kilograms" => "kg",
                "lb" | "lbs" | "pound" | "pounds" => "lb",
                "oz" | "ounce" | "ounces" => "oz",
                "ml" | "millilitre" | "millilitres" | "milliliter" | "milliliters" => "ml",
                "l" | "litre" | "litres" | "liter" | "liters" => "l",
                "piece" | "pieces" | "pc" | "pcs" => "piece",
                "clove" | "cloves" => "clove",
                "slice" | "slices" => "slice",
                "bunch" | "bunches" => "bunch",
                "can" | "cans" => "can",
                "package" | "packages" | "pack" | "packs" | "pkg" => "package",
                "pinch" | "pinches" => "pinch",
                "dash" | "dashes" => "dash",
                "handful" | "handfuls" => "handful",
                _ => continue,
            };
            Some((id.to_owned(), None))
        } else {
            None
        };
        if let Some((id, region)) = resolved {
            let consumed = words[..count].join(" ").len();
            return Some((id, region, consumed));
        }
    }
    None
}
