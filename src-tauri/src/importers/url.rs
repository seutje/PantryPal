use crate::error::{AppError, AppResult};
pub fn validate(raw: &str) -> AppResult<tauri::Url> {
    let invalid = || {
        AppError::validation(
            "Use a public HTTPS recipe URL without credentials or fragments. Only Cook Smarts recipe version queries are supported.",
        )
    };
    if raw.len() > 2048 || raw.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(invalid());
    }
    let url = tauri::Url::parse(raw).map_err(|_| invalid())?;
    let host = url.host_str().ok_or_else(invalid)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || (url.query().is_some() && !super::cook_smarts::version_query(&url))
        || url.fragment().is_some()
        || url.port().is_some()
        || host.parse::<std::net::IpAddr>().is_ok()
        || host.starts_with('[')
        || !host.contains('.')
        || host.ends_with('.')
        || [
            ".localhost",
            ".local",
            ".internal",
            ".test",
            ".invalid",
            ".home",
            ".lan",
        ]
        .iter()
        .any(|suffix| host.ends_with(suffix))
    {
        return Err(invalid());
    }
    Ok(url)
}
pub fn site_message(host: &str) -> &'static str {
    if super::sites::site(host).is_some() {
        return "Authorized website adapter available. Open the recipe, finish any website access prompts, then read the visible recipe for review. Cross-site login may require your regular browser.";
    }
    let belongs = |domain: &str| host == domain || host.ends_with(&format!(".{domain}"));
    if belongs("budgetbytes.com") {
        "Budget Bytes prohibits scraping. Open the page for reference and enter permitted recipe text manually."
    } else if belongs("cooksmarts.com") {
        "Cook Smarts automated importing is unsupported under the reviewed terms. Use permitted manual text or an authorized export."
    } else if belongs("nytimes.com") {
        "NYT Cooking may require a subscription, login or MFA. Use content you can access and are permitted to retain; no paywall bypass is provided."
    } else {
        "No automated site adapter is enabled. Browse normally, then paste authorized JSON-LD or enter recipe text manually. Cross-site login may require your regular browser."
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_dangerous_urls_without_echoing_secrets() {
        for raw in [
            "http://example.com",
            "file:///c:/secret",
            "javascript:alert(1)",
            "https://user:secret@example.com",
            "https://example.com?token=secret",
            "https://example.com/#secret",
            "https://127.0.0.1",
            "https://2130706433",
            "https://[::1]",
            "https://192.168.1.2",
            "https://host.local",
            "https://localhost",
            "https://example.com:8443",
            "https://example.com\n",
        ] {
            let error = validate(raw).unwrap_err();
            assert!(!error.message.contains("secret"));
        }
        assert!(validate(
            "https://www.allrecipes.com/cowboy-mashed-potato-casserole-recipe-8760165"
        )
        .is_ok());
        assert!(site_message("www.budgetbytes.com").contains("adapter available"));
        assert!(!site_message("budgetbytes.com.evil.com").contains("adapter available"));
    }
}
