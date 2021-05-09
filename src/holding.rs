use rust_decimal::prelude::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl Holdings {
    pub fn add_to_currency_holdings(
        mut self: Holdings,
        currency: String,
        amount: Decimal,
        fiat_rate: Decimal,
        date: u64,
        location: Option<String>,
    ) -> Holdings {
        let currency_holding = CurrencyHolding {
            amount,
            rate_in_fiat: fiat_rate,
            date,
            location: location.unwrap_or_else(|| "".to_owned()),
        };

        if let Some(currency_holdings) = self.0.get_mut(&currency) {
            currency_holdings.push(currency_holding);
        } else {
            self.0.insert(currency, vec![currency_holding]);
        }

        self
    }
}
