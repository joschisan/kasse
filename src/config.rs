//! Terminal configuration, resolved from the page URL or the setup screen.
//!
//! Other applications link to the terminal with the recipient baked into the
//! query string:
//!
//! ```text
//! https://joschisan.github.io/kasse/?lnurl=LNURL1...&currency=USD
//! ```
//!
//! When the `lnurl` parameter is missing (or invalid), the app falls back to an
//! in-app setup screen where the operator can enter it manually. `currency`
//! defaults to USD.

use crate::currency::{FiatCurrency, find_fiat_currency};
use crate::payment::parse_lnurl;

const DEFAULT_CURRENCY: &str = "USD";

/// Immutable terminal configuration.
#[derive(Clone)]
pub struct Config {
    /// The resolved LNURL-pay HTTP endpoint.
    pub endpoint: String,
    pub currency: FiatCurrency,
}

/// Values used to prefill the setup screen.
#[derive(Clone, Default)]
pub struct SetupDefaults {
    pub lnurl: String,
    pub currency_code: String,
}

/// What to show on startup.
pub enum Init {
    /// URL carried a valid lnurl — go straight to the terminal.
    Configured(Config),
    /// No (valid) lnurl — show the setup screen, prefilled with anything we got.
    Setup(SetupDefaults),
}

/// Resolve the initial app state from the page URL.
pub fn from_url() -> Init {
    let (lnurl_param, currency_param) = read_params();

    let currency_code = currency_param.unwrap_or_else(|| DEFAULT_CURRENCY.to_string());

    let endpoint = lnurl_param.as_deref().and_then(parse_lnurl);
    let currency = find_fiat_currency(&currency_code);

    if let (Some(endpoint), Some(currency)) = (endpoint.clone(), currency.clone()) {
        return Init::Configured(Config { endpoint, currency });
    }

    Init::Setup(SetupDefaults {
        // Only prefill the field when the code actually parsed; drop an
        // invalid code so the field starts empty.
        lnurl: if endpoint.is_some() {
            lnurl_param.unwrap_or_default()
        } else {
            String::new()
        },
        currency_code: if currency.is_some() {
            currency_code
        } else {
            DEFAULT_CURRENCY.to_string()
        },
    })
}

/// Build a [`Config`] from a raw lnurl string and currency code.
pub fn build_config(lnurl: &str, currency_code: &str) -> Result<Config, String> {
    let endpoint = parse_lnurl(lnurl.trim())
        .ok_or("Enter a valid LNURL or Lightning address".to_string())?;

    let currency = find_fiat_currency(currency_code)
        .ok_or(format!("Unsupported currency '{currency_code}'"))?;

    Ok(Config { endpoint, currency })
}

/// Persist the lnurl + currency into the page URL without reloading, so a
/// refresh restores the terminal instead of dropping back to the setup screen.
pub fn write_url_params(lnurl: &str, currency_code: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let pathname = window.location().pathname().unwrap_or_default();

    let Ok(params) = web_sys::UrlSearchParams::new() else {
        return;
    };
    params.set("lnurl", lnurl.trim());
    params.set("currency", currency_code);

    let query: String = params.to_string().into();
    let url = format!("{pathname}?{query}");

    if let Ok(history) = window.history() {
        let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
}

/// Read `?lnurl=...&currency=...` from the current page URL.
fn read_params() -> (Option<String>, Option<String>) {
    let Some(window) = web_sys::window() else {
        return (None, None);
    };
    let Ok(search) = window.location().search() else {
        return (None, None);
    };
    let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) else {
        return (None, None);
    };

    let nonempty = |s: String| if s.trim().is_empty() { None } else { Some(s) };

    (
        params.get("lnurl").and_then(nonempty),
        params.get("currency").and_then(nonempty),
    )
}
