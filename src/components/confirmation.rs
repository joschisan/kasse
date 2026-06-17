//! Payment confirmation — ported from the cashup Flutter `confirmation_screen.dart`,
//! with an animated success mark.

use leptos::prelude::*;

use crate::config::Config;
use crate::format;
use crate::state::Screen;

#[component]
pub fn Confirmation(
    config: Config,
    amount_fiat: i64,
    amount_msat: i64,
    set_screen: WriteSignal<Screen>,
) -> impl IntoView {
    let fiat = format::fiat(&config.currency.symbol, amount_fiat);
    let sats = format::sats(amount_msat);

    view! {
        <div class="screen confirm">
            <div class="confirm-title">"Payment received!"</div>
            <svg class="success-mark" viewBox="0 0 72 72">
                <circle class="success-ring" cx="36" cy="36" r="34"></circle>
                <circle class="success-fill" cx="36" cy="36" r="34"></circle>
                <path class="success-check" d="M22 38 L31 47 L51 27"></path>
            </svg>
            <div class="confirm-amount amount-block">
                <div class="amount-fiat">{fiat}</div>
                <div class="amount-sub">{sats}</div>
            </div>
            <button
                class="link-accent"
                on:click=move |_| set_screen.set(Screen::Keypad)
            >
                "Continue"
                <svg
                    class="link-arrow"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path d="M5 12h14"></path>
                    <path d="M12 5l7 7-7 7"></path>
                </svg>
            </button>
        </div>
    }
}
