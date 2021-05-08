use rust_decimal::prelude::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TradeWithFiatRate {
    #[serde(rename = "boughtCurrency")]
    pub bought_currency: String,
    #[serde(rename = "soldCurrency")]
    pub sold_currency: String,
    #[serde(rename = "amountSold")]
    pub amount_sold: Decimal,
    pub rate: Decimal,
    pub date: u64,
    #[serde(rename = "exchangeID")]
    pub exchange_id: String,
    pub exchange: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "transactionFee")]
    pub transaction_fee: Decimal,
    #[serde(rename = "transactionFeeCurrency")]
    pub transaction_fee_currency: String,
    #[serde(rename = "fiatRate")]
    pub fiat_rate: Decimal,
}
