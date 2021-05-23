use crate::calculate_gains::calculate_gains;
use crate::holding::Holdings;
use crate::income::Income;
use crate::method::Method;
use crate::trade::Trade;
use rust_decimal::prelude::{Decimal, Zero};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CalculateGainPerTrade {
    pub trades: Vec<Trade>,
    pub holdings: Holdings,
    #[serde(rename = "shortTerm")]
    pub short_term: Decimal,
    #[serde(rename = "longTerm")]
    pub long_term: Decimal,
}

#[wasm_bindgen]
pub fn calculate_gains_per_trade_wasm(
    holdings: &JsValue,
    trade: &JsValue,
    incomes: &JsValue,
    fiat_currency: String,
    method: Method,
) -> JsValue {
    let holdings: Holdings = holdings.into_serde().unwrap();
    let trades: Vec<Trade> = trade.into_serde().unwrap();
    let incomes: Vec<Income> = incomes.into_serde().unwrap();

    JsValue::from_serde(&calculate_gain_per_trade(
        holdings,
        trades,
        incomes,
        fiat_currency,
        method,
    ))
    .unwrap()
}

pub fn calculate_gain_per_trade(
    holdings: Holdings,
    old_trades: Vec<Trade>,
    incomes: Vec<Income>,
    fiat_currency: String,
    method: Method,
) -> CalculateGainPerTrade {
    let mut new_holdings = holdings;
    let mut short_term: Decimal = Zero::zero();
    let mut long_term: Decimal = Zero::zero();
    let mut trades: Vec<Trade> = vec![];

    let mut new_incomes = incomes;

    for trade in old_trades {
        let mut incomes_to_use = vec![];
        while !new_incomes.is_empty() && trade.date > new_incomes[0].date {
            let income: Income = new_incomes.remove(0);
            incomes_to_use.push(income);
        }

        let result = calculate_gains(
            new_holdings,
            vec![trade.clone()],
            incomes_to_use,
            fiat_currency.clone(),
            method,
        );

        new_holdings = result.new_holdings;
        short_term += result.short_term_gain;
        long_term += result.long_term_gain;
        trades.push(Trade {
            short_term: Some(result.short_term_gain),
            long_term: Some(result.long_term_gain),
            ..trade
        });
    }

    let apply_remaining_incomes =
        calculate_gains(new_holdings, vec![], new_incomes, fiat_currency, method);

    CalculateGainPerTrade {
        trades,
        holdings: apply_remaining_incomes.new_holdings,
        short_term,
        long_term,
    }
}

#[cfg(test)]
mod tests {
    use super::calculate_gain_per_trade;
    use crate::calculate_gains::calculate_gains;
    use crate::method::Method;
    use crate::mocks;

    static FIAT_CURRENCY: &str = "FAKE";

    #[test]
    fn calculate_gain_per_trade_identical_to_calculate_gains() {
        let holdings = mocks::mock_holdings(3, 3, None, None);
        let trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);

        let gains = calculate_gains(
            holdings.clone(),
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );
        let gains_per_holding = calculate_gain_per_trade(
            holdings.clone(),
            trades,
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert_eq!(gains.short_term_gain, gains_per_holding.short_term);
        assert_eq!(gains.long_term_gain, gains_per_holding.long_term);
        assert_eq!(gains.new_holdings, gains_per_holding.holdings);
    }
}
