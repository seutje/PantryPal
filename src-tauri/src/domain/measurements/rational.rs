use crate::error::{AppError, AppResult};

/// Reduced, bounded exact arithmetic. Overflow is an error, never rounding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    pub(crate) n: i128,
    pub(crate) d: i128,
}

fn limit() -> AppError {
    AppError::validation("Amount exceeds the supported exact precision. Use a smaller value.")
}
fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
impl Rational {
    pub fn new(n: i128, d: i128) -> AppResult<Self> {
        if d <= 0 || n == i128::MIN {
            return Err(limit());
        }
        let common = gcd(n.abs(), d);
        Ok(Self {
            n: n / common,
            d: d / common,
        })
    }
    pub fn checked_mul(self, other: Self) -> AppResult<Self> {
        let a = gcd(self.n.abs(), other.d);
        let b = gcd(other.n.abs(), self.d);
        Self::new(
            (self.n / a).checked_mul(other.n / b).ok_or_else(limit)?,
            (self.d / b).checked_mul(other.d / a).ok_or_else(limit)?,
        )
    }
    pub fn checked_div(self, other: Self) -> AppResult<Self> {
        if other.n == 0 {
            return Err(limit());
        }
        self.checked_mul(Self::new(other.d * other.n.signum(), other.n.abs())?)
    }
    pub fn checked_add(self, other: Self) -> AppResult<Self> {
        let common = gcd(self.d, other.d);
        Self::new(
            self.n
                .checked_mul(other.d / common)
                .and_then(|n| {
                    other
                        .n
                        .checked_mul(self.d / common)
                        .and_then(|m| n.checked_add(m))
                })
                .ok_or_else(limit)?,
            (self.d / common).checked_mul(other.d).ok_or_else(limit)?,
        )
    }
    pub fn negated(self) -> Self {
        Self {
            n: -self.n,
            d: self.d,
        }
    }
    pub fn fraction(self) -> String {
        if self.d == 1 {
            self.n.to_string()
        } else {
            format!("{}/{}", self.n, self.d)
        }
    }
    pub fn display_fraction(self) -> String {
        if self.d == 1 {
            return self.n.to_string();
        }
        let magnitude = self.n.abs();
        let whole = magnitude / self.d;
        let remainder = magnitude % self.d;
        let glyph = match (remainder, self.d) {
            (1, 2) => Some("½"),
            (1, 3) => Some("⅓"),
            (2, 3) => Some("⅔"),
            (1, 4) => Some("¼"),
            (3, 4) => Some("¾"),
            (1, 8) => Some("⅛"),
            (3, 8) => Some("⅜"),
            (5, 8) => Some("⅝"),
            (7, 8) => Some("⅞"),
            (1, 5) => Some("⅕"),
            (2, 5) => Some("⅖"),
            (3, 5) => Some("⅗"),
            (4, 5) => Some("⅘"),
            (1, 6) => Some("⅙"),
            (5, 6) => Some("⅚"),
            _ => None,
        };
        let sign = if self.n < 0 { "-" } else { "" };
        let prefix = if whole == 0 {
            String::new()
        } else {
            whole.to_string()
        };
        if let Some(glyph) = glyph {
            format!("{sign}{prefix}{glyph}")
        } else {
            format!(
                "{sign}{prefix}{}{remainder}/{}",
                if whole == 0 { "" } else { " " },
                self.d
            )
        }
    }
    /// Round half away from zero, only for display. No floating point.
    pub fn decimal(self, places: u8) -> AppResult<(String, bool)> {
        if places > 6 {
            return Err(AppError::validation("Choose zero to six decimal places."));
        }
        let scale = 10_i128.pow(u32::from(places));
        let scaled = self.n.abs().checked_mul(scale).ok_or_else(limit)?;
        let remainder = scaled % self.d;
        let rounded = scaled / self.d + i128::from(remainder >= self.d / 2 + self.d % 2);
        let sign = if self.n < 0 && rounded != 0 { "-" } else { "" };
        let result = if places == 0 {
            format!("{sign}{rounded}")
        } else {
            let text = format!(
                "{sign}{}.{:0width$}",
                rounded / scale,
                rounded % scale,
                width = usize::from(places)
            );
            text.trim_end_matches('0').trim_end_matches('.').to_string()
        };
        Ok((result, remainder != 0))
    }
}

/// Signed decimal/fraction parser; quantity validation happens at the caller.
/// Unknown wording remains unknown, not zero. Scientific notation/ranges are rejected.
pub fn parse(raw: &str) -> AppResult<Option<Rational>> {
    super::super::validation::text(raw, 120, false)?;
    let text = raw.trim();
    if matches!(
        text.to_lowercase().as_str(),
        "" | "unknown" | "to taste" | "as needed"
    ) {
        return Ok(None);
    }
    let (negative, value) = if let Some(value) = text.strip_prefix('-') {
        (true, value)
    } else {
        (false, text)
    };
    let mut expanded = value.to_string();
    for (glyph, fraction) in [
        ('½', "1/2"),
        ('⅓', "1/3"),
        ('⅔', "2/3"),
        ('¼', "1/4"),
        ('¾', "3/4"),
        ('⅛', "1/8"),
        ('⅜', "3/8"),
        ('⅝', "5/8"),
        ('⅞', "7/8"),
        ('⅕', "1/5"),
        ('⅖', "2/5"),
        ('⅗', "3/5"),
        ('⅘', "4/5"),
        ('⅙', "1/6"),
        ('⅚', "5/6"),
    ] {
        if expanded.ends_with(glyph) {
            expanded.pop();
            if !expanded.trim().is_empty() {
                expanded.push(' ');
            }
            expanded.push_str(fraction);
            break;
        }
    }
    let parts: Vec<_> = expanded.split_whitespace().collect();
    let integer = |s: &str| -> AppResult<i128> {
        if s.is_empty() || s.len() > 10 || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(AppError::validation(
                "Enter a decimal, fraction or mixed fraction.",
            ));
        }
        let value = s.parse::<i128>().map_err(|_| limit())?;
        if value > 1_000_000_000 {
            return Err(limit());
        }
        Ok(value)
    };
    let simple = |s: &str| -> AppResult<Rational> {
        if let Some((n, d)) = s.split_once('/') {
            Rational::new(integer(n)?, integer(d)?)
        } else if let Some((whole, fraction)) = s.split_once('.') {
            if fraction.len() > 9 || fraction.is_empty() {
                return Err(limit());
            }
            let d = 10_i128.pow(fraction.len() as u32);
            Rational::new(
                integer(if whole.is_empty() { "0" } else { whole })? * d + integer(fraction)?,
                d,
            )
        } else {
            Rational::new(integer(s)?, 1)
        }
    };
    let result = match parts.as_slice() {
        [one] => simple(one)?,
        [whole, fraction] if fraction.contains('/') => {
            let f = simple(fraction)?;
            if f.n >= f.d {
                return Err(AppError::validation(
                    "Mixed fraction must have a proper fractional part.",
                ));
            }
            Rational::new(integer(whole)?, 1)?.checked_add(f)?
        }
        _ => {
            return Err(AppError::validation(
                "Enter a decimal, fraction or mixed fraction.",
            ))
        }
    };
    if result.n.abs() > 1_000_000_000 || result.d > 1_000_000_000 {
        return Err(limit());
    }
    Ok(Some(if negative { result.negated() } else { result }))
}
