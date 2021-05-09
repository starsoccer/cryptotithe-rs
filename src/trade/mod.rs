use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
use rust_decimal::prelude::{Decimal, FromPrimitive};
use serde::{Deserialize, Serialize};

mod trade_with_cost_basis;
mod trade_with_fiat_rate;

pub use {trade_with_cost_basis::*, trade_with_fiat_rate::*};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Trade {
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
}

impl Arbitrary<'_> for Trade {
    fn arbitrary(u: &mut Unstructured<'_>) -> ArbitraryResult<Self> {
        Ok(Self {
            bought_currency: String::arbitrary(u)?,
            sold_currency: String::arbitrary(u)?,
            amount_sold: Decimal::from_f64(f64::arbitrary(u)?).unwrap(),
            rate: Decimal::from_f64(f64::arbitrary(u)?).unwrap(),
            date: u64::arbitrary(u)?,
            exchange_id: String::arbitrary(u)?,
            exchange: String::arbitrary(u)?,
            id: String::arbitrary(u)?,
            transaction_fee: Decimal::from_f64(f64::arbitrary(u)?).unwrap(),
            transaction_fee_currency: String::arbitrary(u)?,
        })
    }
}

impl From<TradeWithFiatRate> for Trade {
    fn from(trade: TradeWithFiatRate) -> Self {
        Self {
            bought_currency: trade.bought_currency,
            sold_currency: trade.sold_currency,
            amount_sold: trade.amount_sold,
            rate: trade.rate,
            date: trade.date,
            exchange_id: trade.exchange_id,
            exchange: trade.exchange,
            id: trade.id,
            transaction_fee: trade.transaction_fee,
            transaction_fee_currency: trade.transaction_fee_currency,
        }
    }
}
