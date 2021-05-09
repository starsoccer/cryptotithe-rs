use rust_decimal::prelude::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod add_to_currency_holdings;
pub mod process_trade;

pub use {add_to_currency_holdings::*, process_trade::*};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CurrencyHolding {
    pub amount: Decimal,
    #[serde(rename = "rateInFiat")]
    pub rate_in_fiat: Decimal,
    pub date: u64, // really u32 but bigger then max size
    pub location: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Holdings(pub HashMap<String, Vec<CurrencyHolding>>);