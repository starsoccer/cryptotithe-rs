use crate::holding::Holdings;
use crate::method::Method;
use crate::trade::Trade;
use rust_decimal::prelude::{Decimal, Zero};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CalculateGainPerHolding {
    #[serde(rename = "shortTermTrades")]
    pub short_term_trades: Vec<Trade>,
    #[serde(rename = "longTermTrades")]
    pub long_term_trades: Vec<Trade>,
    #[serde(rename = "shortTermGain")]
    pub short_term_gain: Decimal,
    #[serde(rename = "longTermGain")]
    pub long_term_gain: Decimal,
    #[serde(rename = "shortTermProceeds")]
    pub short_term_proceed: Decimal,
    #[serde(rename = "longTermProceeds")]
    pub long_term_proceed: Decimal,
    #[serde(rename = "shortTermCostBasis")]
    pub short_term_cost_basis: Decimal,
    #[serde(rename = "longTermCostBasis")]
    pub long_term_cost_basis: Decimal,
}

#[wasm_bindgen]
pub fn calculate_gain_per_holdings_wasm(
    holdings: &JsValue,
    trade: &JsValue,
    fiat_currency: String,
    method: Method,
) -> JsValue {
    let holdings: Holdings = holdings.into_serde().unwrap();
    let trades: Vec<Trade> = trade.into_serde().unwrap();

    JsValue::from_serde(&calculate_gain_per_holdings(
        holdings,
        trades,
        fiat_currency,
        method,
    ))
    .unwrap()
}

pub fn calculate_gain_per_holdings(
    holdings: Holdings,
    trades: Vec<Trade>,
    fiat_currency: String,
    method: Method,
) -> CalculateGainPerHolding {
    let mut new_holdings = holdings;
    let mut short_term_gain = Zero::zero();
    let mut short_term_proceed = Zero::zero();
    let mut short_term_cost_basis = Zero::zero();
    let mut long_term_gain = Zero::zero();
    let mut long_term_proceed = Zero::zero();
    let mut long_term_cost_basis = Zero::zero();
    let mut short_term_trades: Vec<Trade> = vec![];
    let mut long_term_trades: Vec<Trade> = vec![];

    for trade in trades {
                // handle this better somewhere else
                if trade.amount_sold > Zero::zero() {
        let result = new_holdings.process_trade(trade, fiat_currency.clone(), method);
                

        short_term_gain += result.short_term_gain;
        short_term_proceed += result.short_term_proceeds;
        short_term_cost_basis += result.short_term_cost_basis;
        long_term_gain += result.long_term_gain;
        long_term_proceed += result.long_term_proceeds;
        long_term_cost_basis += result.long_term_cost_basis;
        new_holdings = result.holdings;

        for cost_basis_trade in result.cost_basis_trades {
            if cost_basis_trade.long_term_trade.unwrap_or(false) {
                long_term_trades.push(cost_basis_trade);
            } else {
                short_term_trades.push(cost_basis_trade);
            }
        }
    }
    }

    CalculateGainPerHolding {
        short_term_trades,
        long_term_trades,
        short_term_gain,
        long_term_gain,
        short_term_proceed,
        long_term_proceed,
        short_term_cost_basis,
        long_term_cost_basis,
    }
}

#[cfg(test)]
mod tests {
    use super::calculate_gain_per_holdings;
    use crate::method::Method;
    use crate::mocks;
    use crate::{QUARTER_IN_MILLISECONDS, YEAR_IN_MILLISECONDS};
    use rust_decimal::prelude::{Decimal, Zero};

    static FIAT_CURRENCY: &str = "FAKE";

    #[test]
    fn calculate_gain_per_trade_identical_to_calculate_gains() {
        let mut holdings = mocks::mock_holdings(1, 5, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0].clone();
        let mut amount = Zero::zero();
        if let Some(currency_holdings) = holdings.0.get_mut(&currency) {
            currency_holdings[1].date = mocks::now_u64() - QUARTER_IN_MILLISECONDS;
            currency_holdings[0].date = mocks::now_u64() - YEAR_IN_MILLISECONDS * 10;
            amount = currency_holdings[0].amount.clone();
        }
        
        let mut trades = mocks::mock_trades(5, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = amount;

        let result = calculate_gain_per_holdings(holdings.clone(), trades.clone(), FIAT_CURRENCY.to_string(), Method::FIFO);

        assert!(!result.short_term_trades.is_empty());
        assert!(!result.long_term_trades.is_empty());

        let mut total_cost_basis = Zero::zero();
        let currency_holdings = holdings.0.get(&currency).unwrap();
        for currency_holding in currency_holdings {
            total_cost_basis += currency_holding.amount * currency_holding.rate_in_fiat;
        }

        assert!(result.short_term_cost_basis + result.long_term_cost_basis < total_cost_basis);

        let mut total_proceeds: Decimal = Zero::zero();
        for trade in trades {
            total_proceeds += trade.amount_sold * trade.fiat_rate();
        }

        assert_eq!((result.short_term_proceed + result.long_term_proceed).round_dp(16), total_proceeds.round_dp(16));
    }
}
