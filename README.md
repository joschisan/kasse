# kasse

A linkable, browser-based Lightning point-of-sale terminal. Configure it entirely
from the URL, so any application can deep-link a merchant straight into a ready-to-use
terminal:

```
https://joschisan.github.io/kasse/?lnurl=LNURL1...&currency=USD
```

The terminal is **stateless** — it holds no funds, stores no payment history, and
transmits nothing beyond the LNURL-pay and exchange-rate requests needed to take a
payment. Everything runs locally in the browser (Rust compiled to WASM via Leptos).

## URL parameters

| Param      | Required | Description                                                        |
| ---------- | -------- | ------------------------------------------------------------------ |
| `lnurl`    | yes      | An LNURL-pay string (`LNURL1...`), a Lightning address (`a@b.com`), or a `lightning:`/`lnurl:`-prefixed value. |
| `currency` | yes      | An ISO fiat currency code, e.g. `USD`, `EUR`, `KES`.               |

## How it works

1. Read `lnurl` + `currency` from the query string.
2. Merchant enters an amount on the keypad.
3. The amount is converted fiat → USD → BTC → msat using the
   [Fedi price feed](https://price-feed.dev.fedibtc.com/latest), and an invoice is
   requested from the LNURL-pay endpoint.
4. The BOLT11 invoice is shown as a QR and the LNURL `verify` URL (LUD-21) is polled
   until the payment settles, validating the preimage against the invoice payment hash.

The payment logic (currency table, fiat→msat conversion, LNURL flow, preimage check)
is ported from the [cashup](https://github.com/joschisan/cashup) Rust core; the
Leptos/Trunk/GitHub-Pages scaffolding follows
[fedimint-walletv2-recoverytool](https://github.com/fedimint/fedimint-walletv2-recoverytool).

## Known limitation: CORS

A browser POS must `fetch()` the merchant's LNURL server and the price feed
cross-origin. Many LNURL implementations send `Access-Control-Allow-Origin: *`, but
not all do — if a provider omits CORS headers, the browser blocks the request. Test
with your actual LNURL provider before relying on it.

## Develop

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk        # or download a release binary
trunk serve                # http://localhost:8080/?lnurl=...&currency=USD
```

On macOS the C parts of `secp256k1` need a wasm-capable clang:

```sh
brew install llvm
CC_wasm32_unknown_unknown=$(brew --prefix llvm)/bin/clang \
AR_wasm32_unknown_unknown=$(brew --prefix llvm)/bin/llvm-ar \
trunk serve
```

## Deploy

Pushing to `main` triggers `.github/workflows/deploy.yml`, which builds with
`trunk build --release --public-url /kasse/` and publishes to GitHub Pages.
Enable Pages under **Settings → Pages → Source: GitHub Actions**.

## License

MIT
