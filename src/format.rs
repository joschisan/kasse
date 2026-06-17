//! Small display formatting helpers.

/// Group an integer with thousands separators: `1234567` → `1,234,567`.
pub fn group_int(n: i64) -> String {
    let negative = n < 0;
    let digits = n.abs().to_string();
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if negative { format!("-{out}") } else { out }
}

/// Format a fiat amount in minor units: `(\"$\", 123456)` → `$ 123,456.00`.
pub fn fiat(symbol: &str, minor_units: i64) -> String {
    let whole = minor_units / 100;
    let cents = (minor_units % 100).abs();
    format!("{} {}.{:02}", symbol, group_int(whole), cents)
}

/// Format a millisatoshi amount as grouped sats: `12840000` → `12,840 sats`.
pub fn sats(amount_msat: i64) -> String {
    format!("{} sats", group_int(amount_msat / 1000))
}
