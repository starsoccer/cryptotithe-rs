use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
use rust_decimal::prelude::{Decimal, FromPrimitive, Zero};
use serde::{Deserialize, Serialize};

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
    #[serde(rename = "fiatRate")]
    pub fiat_rate: Option<Decimal>,
    #[serde(rename = "shortTerm")]
    pub short_term: Option<Decimal>,
    #[serde(rename = "longTerm")]
    pub long_term: Option<Decimal>,
    #[serde(rename = "dateAcquired")]
    pub date_acquired: Option<u64>,
    #[serde(rename = "costBasis")]
    pub cost_basis: Option<Decimal>,
    #[serde(rename = "longtermTrade")]
    pub long_term_trade: Option<bool>,
}

impl Trade {
    pub fn fiat_rate(&self) -> Decimal {
        self.fiat_rate.unwrap_or_else(Zero::zero)
    }

    pub fn cost_basis(&self) -> Decimal {
        self.cost_basis.unwrap_or_else(Zero::zero)
    }
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
            fiat_rate: None,
            short_term: None,
            long_term: None,
            date_acquired: None,
            cost_basis: None,
            long_term_trade: None,
        })
    }
}
