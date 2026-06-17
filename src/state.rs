//! The terminal's screen state machine.
//!
//! Mirrors the screen flow of the cashup Flutter app (amount → invoice →
//! confirmation), modeled as a typed enum like the recovery tool's
//! `WizardState`.

use crate::payment::ResolvedInvoice;

#[derive(Clone)]
pub enum Screen {
    /// Numeric keypad for entering the fiat amount.
    Keypad,
    /// Displaying the invoice QR and polling for settlement.
    Invoice { resolved: ResolvedInvoice },
    /// Payment settled.
    Confirmed { amount_fiat: i64, amount_msat: i64 },
}
