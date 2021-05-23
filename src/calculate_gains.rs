use crate::holding::Holdings;
use crate::income::Income;
use crate::method::Method;
use crate::trade::Trade;
use rust_decimal::prelude::{Decimal, Zero};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CalculateGains {
    #[serde(rename = "newHoldings")]
    pub new_holdings: Holdings,
    #[serde(rename = "longTermGain")]
    pub long_term_gain: Decimal,
    #[serde(rename = "shortTermGain")]
    pub short_term_gain: Decimal,
}

#[wasm_bindgen]
pub fn calculate_gains_wasm(
    holdings: &JsValue,
    trade: &JsValue,
    incomes: &JsValue,
    fiat_currency: String,
    method: Method,
) -> JsValue {
    let holdings: Holdings = holdings.into_serde().unwrap();
    let trades: Vec<Trade> = trade.into_serde().unwrap();
    let incomes: Vec<Income> = incomes.into_serde().unwrap();
    JsValue::from_serde(&calculate_gains(
        holdings,
        trades,
        incomes,
        fiat_currency,
        method,
    ))
    .unwrap()
}

pub fn calculate_gains(
    holdings: Holdings,
    trades: Vec<Trade>,
    incomes: Vec<Income>,
    fiat_currency: String,
    method: Method,
) -> CalculateGains {
    let mut short_term_gain = Zero::zero();
    let mut long_term_gain = Zero::zero();
    let mut new_holdings = holdings;

    let mut incomes_to_apply = incomes;

    for trade in trades {
        while !incomes_to_apply.is_empty() && trade.date > incomes_to_apply[0].date {
            let income = incomes_to_apply.remove(0);
            new_holdings = new_holdings.add_to_currency_holdings(
                income.currency.clone(),
                income.amount,
                income.clone().fiat_rate(),
                income.date,
                None,
            );
        }

        // handle this better somewhere else
        if trade.amount_sold > Zero::zero() {
            let result = new_holdings.process_trade(trade.clone(), fiat_currency.clone(), method);
            short_term_gain += result.short_term_gain;
            long_term_gain += result.long_term_gain;
            new_holdings = result.holdings;
        }
    }

    for income in incomes_to_apply {
        new_holdings = new_holdings.add_to_currency_holdings(
            income.currency.clone(),
            income.amount,
            income.clone().fiat_rate(),
            income.date,
            None,
        );
    }

    CalculateGains {
        short_term_gain,
        long_term_gain,
        new_holdings,
    }
}

#[cfg(test)]
mod tests {
    use super::calculate_gains;
    use crate::method::Method;
    use crate::mocks;
    use crate::{QUARTER_IN_MILLISECONDS, YEAR_IN_MILLISECONDS};
    use rust_decimal::prelude::Zero;
    use rust_decimal_macros::*;

    static FIAT_CURRENCY: &str = "FAKE";

    #[test]
    fn single_holding_single_trade_short_term_no_overflow() {
        let holdings =
            mocks::mock_holdings(1, 1, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = currency_holdings[0].amount;
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert!(result.long_term_gain.is_zero());

        let bought_currency_holdings = result
            .new_holdings
            .0
            .get(&trades[0].bought_currency)
            .expect("bought currency not found");
        assert!(!bought_currency_holdings.is_empty());
        assert_eq!(
            result.short_term_gain,
            (trades[0].fiat_rate() - currency_holdings[0].rate_in_fiat) * trades[0].amount_sold
        );
    }

    #[test]
    fn single_holding_single_trade_short_term_overflow() {
        let holdings =
            mocks::mock_holdings(1, 1, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = currency_holdings[0].amount * dec!(2);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert!(result.long_term_gain.is_zero());

        let bought_currency_holdings = result
            .new_holdings
            .0
            .get(&trades[0].bought_currency)
            .expect("bought currency not found");
        assert!(!bought_currency_holdings.is_empty());
        assert_eq!(
            result.short_term_gain,
            (trades[0].fiat_rate() - currency_holdings[0].rate_in_fiat)
                * currency_holdings[0].amount
                + trades[0].fiat_rate() * (trades[0].amount_sold - currency_holdings[0].amount)
        );
    }

    #[test]
    fn single_holding_single_trade_long_term_no_overflow() {
        let holdings =
            mocks::mock_holdings(1, 1, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = currency_holdings[0].amount;
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert!(result.short_term_gain.is_zero());

        let bought_currency_holdings = result
            .new_holdings
            .0
            .get(&trades[0].bought_currency)
            .expect("bought currency not found");
        assert!(!bought_currency_holdings.is_empty());
        assert_eq!(
            result.long_term_gain,
            (trades[0].fiat_rate() - currency_holdings[0].rate_in_fiat) * trades[0].amount_sold
        );
    }

    #[test]
    fn single_holding_single_trade_long_term_overflow() {
        let holdings =
            mocks::mock_holdings(1, 1, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), true);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        let bought_currency_holdings = result
            .new_holdings
            .0
            .get(&trades[0].bought_currency)
            .expect("bought currency not found");
        assert!(!bought_currency_holdings.is_empty());

        assert_eq!(
            result.short_term_gain,
            trades[0].fiat_rate() * (trades[0].amount_sold - currency_holdings[0].amount)
        );
        assert_eq!(
            result.long_term_gain,
            (trades[0].fiat_rate() - currency_holdings[0].rate_in_fiat)
                * currency_holdings[0].amount
        );
    }

    #[test]
    fn single_holding_multiple_trade_short_term_no_overflow() {
        let holdings =
            mocks::mock_holdings(1, 1, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(5, mocks::now_u64(), holdings.clone(), false);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert!(result.long_term_gain.is_zero());

        let mut gain = Zero::zero();
        for trade in trades {
            gain += (trade.fiat_rate() - currency_holdings[0].rate_in_fiat) * trade.amount_sold;
        }

        assert_eq!(result.short_term_gain, gain);
    }

    #[test]
    fn single_holding_multiple_trade_short_term_overflow() {
        let holdings =
            mocks::mock_holdings(1, 1, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(5, mocks::now_u64(), holdings.clone(), true);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert!(result.long_term_gain.is_zero());

        let mut gain = Zero::zero();
        let mut used_holding_amount = currency_holdings[0].amount;
        for trade in trades {
            if used_holding_amount.is_zero() {
                gain += trade.fiat_rate() * trade.amount_sold;
            } else if used_holding_amount >= trade.amount_sold {
                gain += (trade.fiat_rate() - currency_holdings[0].rate_in_fiat) * trade.amount_sold;
                used_holding_amount -= trade.amount_sold;
            } else {
                gain +=
                    (trade.fiat_rate() - currency_holdings[0].rate_in_fiat) * used_holding_amount;
                gain += trade.fiat_rate() * (trade.amount_sold - used_holding_amount);
                used_holding_amount = Zero::zero();
            }
        }

        assert_eq!(result.short_term_gain, gain);
    }

    #[test]
    fn single_holding_multiple_trade_long_term_no_overflow() {
        let holdings =
            mocks::mock_holdings(1, 1, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(5, mocks::now_u64(), holdings.clone(), false);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert!(result.short_term_gain.is_zero());

        let mut gain = Zero::zero();
        for trade in trades {
            gain += (trade.fiat_rate() - currency_holdings[0].rate_in_fiat) * trade.amount_sold;
        }

        assert_eq!(result.long_term_gain, gain);
    }

    #[test]
    fn single_holding_multiple_trade_long_term_overflow() {
        let holdings =
            mocks::mock_holdings(1, 1, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(5, mocks::now_u64(), holdings.clone(), true);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        let mut gain = Zero::zero();
        let mut used_holding_amount = currency_holdings[0].amount;
        for trade in trades {
            if used_holding_amount.is_zero() {
                gain += trade.fiat_rate() * trade.amount_sold;
            } else if used_holding_amount >= trade.amount_sold {
                gain += (trade.fiat_rate() - currency_holdings[0].rate_in_fiat) * trade.amount_sold;
                used_holding_amount -= trade.amount_sold;
            } else {
                gain +=
                    (trade.fiat_rate() - currency_holdings[0].rate_in_fiat) * used_holding_amount;
                gain += trade.fiat_rate() * (trade.amount_sold - used_holding_amount);
                used_holding_amount = Zero::zero();
            }
        }

        assert_eq!(result.long_term_gain + result.short_term_gain, gain);
    }

    #[test]
    fn multiple_holding_single_trade_short_term_no_overflow() {
        let holdings =
            mocks::mock_holdings(1, 5, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert!(result.long_term_gain.is_zero());

        let mut gain = Zero::zero();
        let mut amount_left = trades[0].amount_sold;
        for holding in currency_holdings {
            if amount_left > Zero::zero() {
                let amount_to_deduct = amount_left.min(holding.amount);
                amount_left -= amount_to_deduct;
                gain += (trades[0].fiat_rate() - holding.rate_in_fiat) * amount_to_deduct;
            }
        }

        assert_eq!(result.short_term_gain, gain);
    }

    #[test]
    fn multiple_holding_single_trade_short_term_overflow() {
        let holdings =
            mocks::mock_holdings(1, 5, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), true);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert!(result.long_term_gain.is_zero());

        let mut gain = Zero::zero();
        let mut amount_left = trades[0].amount_sold;
        for holding in currency_holdings {
            if amount_left > Zero::zero() {
                let amount_to_deduct = amount_left.min(holding.amount);
                amount_left -= amount_to_deduct;
                gain += (trades[0].fiat_rate() - holding.rate_in_fiat) * amount_to_deduct;
            }
        }

        gain += amount_left * trades[0].fiat_rate();

        assert_eq!(result.short_term_gain, gain);
    }

    #[test]
    fn multiple_holding_single_trade_long_term_no_overflow() {
        let holdings =
            mocks::mock_holdings(1, 5, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        assert!(result.short_term_gain.is_zero());

        let mut gain = Zero::zero();
        let mut amount_left = trades[0].amount_sold;
        for holding in currency_holdings {
            if amount_left > Zero::zero() {
                let amount_to_deduct = amount_left.min(holding.amount);
                amount_left -= amount_to_deduct;
                gain += (trades[0].fiat_rate() - holding.rate_in_fiat) * amount_to_deduct;
            }
        }

        assert_eq!(result.long_term_gain, gain);
    }

    #[test]
    fn multiple_holding_single_trade_long_term_overflow() {
        let holdings =
            mocks::mock_holdings(1, 5, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), true);
        let result = calculate_gains(
            holdings,
            trades.clone(),
            vec![],
            FIAT_CURRENCY.to_string(),
            Method::FIFO,
        );

        let mut gain = Zero::zero();
        let mut amount_left = trades[0].amount_sold;
        for holding in currency_holdings {
            if amount_left > Zero::zero() {
                let amount_to_deduct = amount_left.min(holding.amount);
                amount_left -= amount_to_deduct;
                gain += (trades[0].fiat_rate() - holding.rate_in_fiat) * amount_to_deduct;
            }
        }

        assert_eq!(result.short_term_gain, amount_left * trades[0].fiat_rate());
        assert_eq!(result.long_term_gain, gain);
    }
}
