//! Invoice QR + payment verification — ported from the cashup Flutter
//! `invoice_screen.dart`.

use leptos::prelude::*;
use qrcode::QrCode;
use qrcode::render::svg;

use crate::config::Config;
use crate::format;
use crate::payment::ResolvedInvoice;
use crate::state::Screen;

fn render_qr(data: &str) -> String {
    match QrCode::new(data.to_uppercase().as_bytes()) {
        Ok(code) => code
            .render::<svg::Color>()
            .min_dimensions(320, 320)
            .quiet_zone(false)
            .build(),
        Err(_) => "<p>Failed to render QR code</p>".to_string(),
    }
}

#[component]
pub fn Invoice(
    config: Config,
    resolved: ResolvedInvoice,
    set_screen: WriteSignal<Screen>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let amount_fiat = resolved.amount_fiat;
    let amount_msat = resolved.amount_msat();
    let qr_svg = render_qr(&resolved.raw());

    // Start polling the LNURL verify URL on mount.
    let invoice = resolved.invoice.clone();
    let verify = resolved.verify.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match crate::payment::verify_payment(&invoice, &verify).await {
            Ok(()) => set_screen.set(Screen::Confirmed {
                amount_fiat,
                amount_msat,
            }),
            Err(e) => {
                set_error.set(Some(e));
                set_screen.set(Screen::Keypad);
            }
        }
    });

    let fiat = format::fiat(&config.currency.symbol, amount_fiat);
    let sats = format::sats(amount_msat);

    view! {
        <div class="screen invoice">
            <div class="qr" inner_html=qr_svg></div>
            <div class="amount-block">
                <div class="amount-fiat">{fiat}</div>
                <div class="amount-sub">{sats}</div>
            </div>
            <div class="waiting">
                <span class="pulse-dot"></span>
                "Waiting for payment"
            </div>
            <button
                class="btn btn-ghost"
                on:click=move |_| set_screen.set(Screen::Keypad)
            >
                "Cancel"
            </button>
        </div>
    }
}
