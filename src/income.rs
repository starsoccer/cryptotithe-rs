use rust_decimal::prelude::{Decimal, Zero};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Income {
    pub amount: Decimal,
    pub currency: String,
    #[serde(rename = "transactionID")]
    pub transaction_id: Option<String>,
    #[serde(rename = "ID")]
    pub id: String,
    pub fee: Option<Decimal>,
    pub date: u64,
    #[serde(rename = "fiatRate")]
    pub fiat_rate: Option<Decimal>
}

impl Income {
    pub fn fiat_rate (self: Income) -> Decimal {
        self.fiat_rate.unwrap_or_else(Zero::zero)
    }
}