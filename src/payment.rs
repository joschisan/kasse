//! LNURL-pay resolution and verification.
//!
//! Ported from the cashup Rust core (`cashup/rust/src/lib.rs`). The LNURL
//! protocol calls reuse the same `fedimint-lnurl` crate. The only changes for
//! the browser/WASM target: the SQLite persistence and the `Arc<Mutex>` caches
//! are dropped, `tokio::time::sleep` becomes `gloo_timers`, and `Instant` timing
//! becomes `js_sys::Date::now()`.

use std::collections::BTreeMap;

use bitcoin_hashes::Hash;
use bitcoin_hashes::sha256;
use gloo_timers::future::TimeoutFuture;
use lightning_invoice::Bolt11Invoice;
use serde::Deserialize;

/// A resolved LNURL invoice ready to be displayed and verified.
#[derive(Clone)]
pub struct ResolvedInvoice {
    pub invoice: Bolt11Invoice,
    pub verify: String,
    pub amount_fiat: i64,
}

impl ResolvedInvoice {
    pub fn raw(&self) -> String {
        self.invoice.to_string()
    }

    pub fn amount_msat(&self) -> i64 {
        self.invoice.amount_milli_satoshis().unwrap() as i64
    }
}

/// Parse an LNURL string into its HTTP endpoint.
///
/// Ported verbatim from cashup's `parse_lnurl`, returning the endpoint string
/// instead of an opaque wrapper. Handles `lightning:`/`lnurl:` prefixes, LNURLs
/// embedded in URL query parameters, bech32 LNURLs, and Lightning addresses.
pub fn parse_lnurl(lnurl: &str) -> Option<String> {
    if let Some(stripped) = lnurl.strip_prefix("lightning:") {
        return parse_lnurl(stripped);
    }

    if let Some(stripped) = lnurl.strip_prefix("lnurl:") {
        return parse_lnurl(stripped);
    }

    // Try to parse as URL and extract LNURL from query parameters
    if let Ok(url) = reqwest::Url::parse(&lnurl.to_lowercase()) {
        for (key, value) in url.query_pairs() {
            if key == "lightning" || key == "lnurl" {
                if let Some(result) = parse_lnurl(&value) {
                    return Some(result);
                }
            }
        }
    }

    if let Some(endpoint) = fedimint_lnurl::parse_lnurl(lnurl) {
        return Some(endpoint);
    }

    if let Some(endpoint) = fedimint_lnurl::parse_address(lnurl) {
        return Some(endpoint);
    }

    None
}

/// Resolve a fiat amount (in minor units) into a payable Lightning invoice.
pub async fn resolve(
    endpoint: &str,
    currency_code: &str,
    amount_fiat: i64,
) -> Result<ResolvedInvoice, String> {
    let exchange_response = fetch_exchange_rate().await?;

    let pay_response = fedimint_lnurl::request(endpoint)
        .await
        .map_err(|e| format!("Failed to fetch LNURL response: {e}"))?;

    let (invoice, verify) = resolve_amount_with_currency_code(
        exchange_response,
        pay_response,
        currency_code.to_string(),
        amount_fiat,
    )
    .await?;

    Ok(ResolvedInvoice {
        invoice,
        verify,
        amount_fiat,
    })
}

/// Poll the LNURL `verify` URL until the invoice settles or expires.
///
/// Ported from cashup's `verification_task`, minus the SQLite write.
pub async fn verify_payment(invoice: &Bolt11Invoice, verify: &str) -> Result<(), String> {
    let payment_hash = *invoice.payment_hash();
    let expiry_ms = invoice.expiry_time().as_millis() as f64;
    let start_time = js_sys::Date::now();

    while js_sys::Date::now() - start_time < expiry_ms {
        match fedimint_lnurl::verify_invoice(verify).await {
            Ok(response) => {
                if response.settled {
                    let preimage = response
                        .preimage
                        .ok_or("Response is missing preimage".to_string())?;

                    if sha256::Hash::hash(&preimage) != payment_hash {
                        return Err("Response preimage hash is invalid".to_string());
                    }

                    return Ok(());
                }

                TimeoutFuture::new(1000).await;
            }
            Err(_) => TimeoutFuture::new(10000).await,
        }
    }

    Err("Invoice expired".to_string())
}

#[derive(Deserialize, Clone)]
struct FediPriceResponse {
    prices: BTreeMap<String, ExchangeRate>,
}

#[derive(Deserialize, Clone)]
struct ExchangeRate {
    rate: f64,
}

async fn fetch_exchange_rate() -> Result<FediPriceResponse, String> {
    reqwest::get("https://price-feed.dev.fedibtc.com/latest")
        .await
        .map_err(|_| "Failed to fetch exchange rates".to_string())?
        .json::<FediPriceResponse>()
        .await
        .map_err(|_| "Failed to parse exchange rates".to_string())
}

/// Convert a fiat amount in minor units into msat and request an invoice.
///
/// Ported verbatim from cashup's `resolve_amount_with_currency_code`.
async fn resolve_amount_with_currency_code(
    exchange_response: FediPriceResponse,
    pay_response: fedimint_lnurl::PayResponse,
    currency_code: String,
    amount_fiat: i64,
) -> Result<(Bolt11Invoice, String), String> {
    // Step 1: Convert minor units to major units (e.g., 1234 cents → 12.34 EUR)
    let amount_fiat = amount_fiat as f64 / 100.0;

    // Step 2: Convert currency to USD (via exchange rate)
    let amount_in_usd = if currency_code == "USD" {
        amount_fiat
    } else {
        let currency_to_usd_rate = exchange_response
            .prices
            .get(&format!("{currency_code}/USD"))
            .ok_or("Selected currency not supported".to_string())?
            .rate;

        amount_fiat * currency_to_usd_rate
    };

    // Step 3: Convert USD to BTC
    let usd_to_btc_rate = exchange_response
        .prices
        .get("BTC/USD")
        .ok_or("BTC/USD rate not found".to_string())?
        .rate;

    let amount_in_btc = amount_in_usd / usd_to_btc_rate;

    // Step 4: Convert BTC to millisatoshis (1 BTC = 100,000,000,000 msat) but
    // rounded to full satoshis to be compatible with blink's api
    let amount_msat = (amount_in_btc * 100_000_000.0).round() as u64 * 1000;

    let invoice_response = fedimint_lnurl::get_invoice(&pay_response, amount_msat)
        .await
        .map_err(|e| format!("Failed to get invoice: {e}"))?;

    let verify = invoice_response
        .verify
        .ok_or("Invoice response is missing verify URL".to_string())?;

    if invoice_response.pr.amount_milli_satoshis().is_none() {
        return Err("Invoice amount is not set".to_string());
    }

    Ok((invoice_response.pr, verify))
}

// Silence the unused-crate-dependency lint for getrandom shims that are only
// pulled in to satisfy transitive wasm requirements.
#[allow(unused_imports)]
use {getrandom as _, getrandom_0_2 as _};

#[cfg(test)]
mod tests {
    use super::parse_lnurl;

    #[test]
    fn decodes_canonical_bech32_lnurl() {
        // Canonical example from the LNURL spec.
        let lnurl = "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";
        let endpoint = parse_lnurl(lnurl).expect("should decode");
        assert!(
            endpoint.starts_with("https://service.com/api"),
            "got {endpoint}"
        );
    }

    #[test]
    fn resolves_lightning_address() {
        let endpoint = parse_lnurl("alice@example.com").expect("should resolve");
        assert_eq!(
            endpoint,
            "https://example.com/.well-known/lnurlp/alice"
        );
    }

    #[test]
    fn strips_lightning_prefix() {
        let endpoint = parse_lnurl("lightning:alice@example.com").expect("should resolve");
        assert_eq!(
            endpoint,
            "https://example.com/.well-known/lnurlp/alice"
        );
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_lnurl("not-an-lnurl").is_none());
    }
}
