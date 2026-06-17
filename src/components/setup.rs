//! Setup screen — shown when no (valid) lnurl is present in the URL. Lets the
//! operator enter a Lightning address / LNURL and pick a currency (default USD).

use leptos::prelude::*;

use crate::config::{self, Config, SetupDefaults};
use crate::currency::list_currencies;

#[component]
pub fn Setup(defaults: SetupDefaults, set_config: WriteSignal<Option<Config>>) -> impl IntoView {
    let (lnurl, set_lnurl) = signal(defaults.lnurl.clone());
    let (currency, set_currency) = signal(defaults.currency_code.clone());
    let (error, set_error) = signal(Option::<String>::None);

    let selected_code = defaults.currency_code.clone();
    let options = list_currencies()
        .into_iter()
        .map(|c| {
            let selected = c.code == selected_code;
            let label = format!("{} ({})", c.name, c.code);
            view! {
                <option value=c.code.clone() selected=selected>
                    {label}
                </option>
            }
        })
        .collect::<Vec<_>>();

    let on_continue = move |_| {
        let lnurl = lnurl.get_untracked();
        let currency = currency.get_untracked();
        match config::build_config(&lnurl, &currency) {
            Ok(cfg) => {
                // Persist the recipient into the URL so a refresh stays on the
                // terminal instead of returning to setup.
                config::write_url_params(&lnurl, &currency);
                set_config.set(Some(cfg));
            }
            Err(e) => set_error.set(Some(e)),
        }
    };

    view! {
        <div class="setup">
            <div class="setup-head">
                <div class="setup-title">"Setup Kasse"</div>
                <p class="setup-help">
                    "no funds are held on this device"
                </p>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <div class="field">
                <label class="field-label" for="lnurl-input">"Lightning Url"</label>
                <input
                    id="lnurl-input"
                    class="input input-code"
                    type="text"
                    placeholder="LNURL1…"
                    autocomplete="off"
                    autocapitalize="off"
                    spellcheck="false"
                    prop:value=move || lnurl.get()
                    on:input=move |ev| set_lnurl.set(event_target_value(&ev))
                />
            </div>

            <div class="field">
                <label class="field-label" for="currency-select">"Currency"</label>
                <select
                    id="currency-select"
                    class="select"
                    on:change=move |ev| set_currency.set(event_target_value(&ev))
                >
                    {options}
                </select>
            </div>

            <div class="setup-actions">
                <button
                    class="btn btn-accent btn-block"
                    disabled=move || lnurl.get().trim().is_empty()
                    on:click=on_continue
                >
                    "Confirm"
                </button>
            </div>
        </div>
    }
}
