mod components;
mod config;
mod currency;
mod format;
mod payment;
mod state;

use leptos::prelude::*;

use crate::components::{Confirmation, Invoice, Keypad, Setup};
use crate::config::{Config, Init, SetupDefaults};
use crate::state::Screen;

/// Root component: resolve the URL configuration, otherwise show the setup
/// screen until a config is entered, then run the terminal.
#[component]
fn App() -> impl IntoView {
    let (defaults, initial) = match config::from_url() {
        Init::Configured(config) => (SetupDefaults::default(), Some(config)),
        Init::Setup(defaults) => (defaults, None),
    };

    let (config, set_config) = signal(initial);

    move || match config.get() {
        Some(config) => view! { <Terminal config=config /> }.into_any(),
        None => view! {
            <div class="app">
                <div class="card">
                    <Setup defaults=defaults.clone() set_config=set_config />
                </div>
            </div>
        }
        .into_any(),
    }
}

/// The configured terminal — a small screen state machine.
#[component]
fn Terminal(config: Config) -> impl IntoView {
    let (screen, set_screen) = signal(Screen::Keypad);
    let (error, set_error) = signal(Option::<String>::None);

    let subtitle = format!("{} · {}", config.currency.name, config.currency.code);

    view! {
        <div class="app">
            <div class="card">
                {
                    let subtitle = subtitle.clone();
                    move || (!matches!(screen.get(), Screen::Confirmed { .. }))
                        .then(|| view! { <p class="subtitle">{subtitle.clone()}</p> })
                }

                <Show when=move || error.get().is_some()>
                    <div class="error">{move || error.get().unwrap_or_default()}</div>
                </Show>

                {move || {
                    let config = config.clone();
                    match screen.get() {
                        Screen::Keypad => view! {
                            <Keypad config=config set_screen=set_screen set_error=set_error />
                        }.into_any(),
                        Screen::Invoice { resolved } => view! {
                            <Invoice
                                config=config
                                resolved=resolved
                                set_screen=set_screen
                                set_error=set_error
                            />
                        }.into_any(),
                        Screen::Confirmed { amount_fiat, amount_msat } => view! {
                            <Confirmation
                                config=config
                                amount_fiat=amount_fiat
                                amount_msat=amount_msat
                                set_screen=set_screen
                            />
                        }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    // Remove the loading spinner now that wasm is ready
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("loading"))
    {
        el.remove();
    }
    leptos::mount::mount_to_body(App);
}
