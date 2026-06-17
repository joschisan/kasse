//! Amount entry — ported from the cashup Flutter `amount_screen.dart`.

use leptos::prelude::*;

use crate::config::Config;
use crate::format;
use crate::state::Screen;

#[component]
pub fn Keypad(
    config: Config,
    set_screen: WriteSignal<Screen>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (amount, set_amount) = signal(0i64);
    let (resolving, set_resolving) = signal(false);

    let endpoint = config.endpoint.clone();
    let code = config.currency.code.clone();
    let symbol = config.currency.symbol.clone();

    // Accumulate minor units, capped like the Flutter keypad.
    let on_digit = move |d: i64| {
        set_amount.update(|a| {
            if *a <= 9_999_999_999 {
                *a = *a * 10 + d;
            }
        });
    };

    let on_continue = move |_| {
        if resolving.get_untracked() {
            return;
        }
        let amount_fiat = amount.get_untracked();
        if amount_fiat == 0 {
            set_error.set(Some("Please enter an amount".to_string()));
            return;
        }
        set_error.set(None);
        set_resolving.set(true);

        let endpoint = endpoint.clone();
        let code = code.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match crate::payment::resolve(&endpoint, &code, amount_fiat).await {
                Ok(resolved) => set_screen.set(Screen::Invoice { resolved }),
                Err(e) => {
                    set_error.set(Some(e));
                    set_resolving.set(false);
                }
            }
        });
    };

    let display = {
        let symbol = symbol.clone();
        move || format::fiat(&symbol, amount.get())
    };

    let digit_buttons = (1..=9)
        .map(|d| {
            view! {
                <button
                    class="key"
                    disabled=move || resolving.get()
                    on:click=move |_| on_digit(d)
                >
                    {d}
                </button>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="screen">
            <div class="amount">
                <div class="amount-fiat">{display}</div>
            </div>

            <button
                class="btn btn-accent btn-block charge-btn"
                disabled=move || resolving.get()
                on:click=on_continue
            >
                <Show
                    when=move || resolving.get()
                    fallback=|| view! { "Continue" }
                >
                    <span class="spinner spinner-sm spinner-light"></span>
                    "Generating Invoice"
                </Show>
            </button>

            <div class="keypad">
                {digit_buttons}
                <button
                    class="key key-action"
                    disabled=move || resolving.get()
                    on:click=move |_| set_amount.set(0)
                >
                    "✕"
                </button>
                <button
                    class="key"
                    disabled=move || resolving.get()
                    on:click=move |_| on_digit(0)
                >
                    "0"
                </button>
                <button
                    class="key key-action"
                    disabled=move || resolving.get()
                    on:click=move |_| set_amount.update(|a| *a /= 10)
                >
                    "⌫"
                </button>
            </div>
        </div>
    }
}
