use crate::holding::Holdings;
use crate::trade::Trade;
use crate::method::Method;
use rust_decimal::prelude::Decimal;
use rust_decimal_macros::*;

pub struct CalculateGains {
    new_holdings: Holdings,
    long_term_gain: Decimal,
    short_term_gain: Decimal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Income {
    pub amount: Decimal,
    pub currency: String,
    pub transaction_id: String,
    pub id: String,
    pub fee: Option<Decimal>,
    pub date: u64,
    pub fiat_rate: Option<Decimal>
}

impl Income {
    fn fiat_rate (self: Income) -> Decimal {
        self.fiat_rate.unwrap_or_else(|| dec!(0))
    }
}

pub fn calculate_gains (
    holdings: Holdings,
    trades: Vec<Trade>,
    incomes: Vec<Income>,
    fiat_currency: String,
    method: Method,
) -> CalculateGains {
    let mut short_term_gain = dec!(0);
    let mut long_term_gain = dec!(0);
    let mut new_holdings = holdings;

    let mut incomes_to_apply = incomes;

    for trade in trades {
        while !incomes_to_apply.is_empty() && trade.date > incomes_to_apply[0].date {
            let income = incomes_to_apply.remove(0);
            new_holdings = new_holdings.add_to_currency_holdings(income.currency.clone(), income.amount, income.clone().fiat_rate(), income.date, None);
        }

        let result = new_holdings.process_trade(trade.clone(), fiat_currency.clone(), method);
        short_term_gain += result.short_term_gain;
        long_term_gain += result.long_term_gain;
        new_holdings = result.holdings;
    }

    for income in incomes_to_apply {
        new_holdings = new_holdings.add_to_currency_holdings(income.currency.clone(), income.amount, income.clone().fiat_rate(), income.date, None);
    }

    CalculateGains {
        short_term_gain,
        long_term_gain,
        new_holdings,
    }
}

#[cfg(test)]
mod tests {
    use crate::method::Method;
    use crate::mocks;
    use crate::{QUARTER_IN_MILLISECONDS, YEAR_IN_MILLISECONDS, trade::Trade, holding::Holdings};
    use rust_decimal::prelude::{Decimal, Zero};
    use rust_decimal_macros::*;
    use super::calculate_gains;

    static FIAT_CURRENCY: &str = "FAKE";

    #[test]
    fn single_holding_single_trade_short_term_no_overflow() {
        let holdings = mocks::mock_holdings(1, 1, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = currency_holdings[0].amount;
        let result = calculate_gains(holdings, trades.clone(), vec!(), FIAT_CURRENCY.to_string(), Method::FIFO);

        assert!(result.long_term_gain.is_zero());

        let bought_currency_holdings = result.new_holdings.0.get(&trades[0].bought_currency).expect("bought currency not found");
        assert!(!bought_currency_holdings.is_empty());
        assert_eq!(result.short_term_gain, (trades[0].fiat_rate() - currency_holdings[0].rate_in_fiat) * trades[0].amount_sold);
    }

    #[test]
    fn single_holding_single_trade_short_term_overflow() {
        let holdings = mocks::mock_holdings(1, 1, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = currency_holdings[0].amount * dec!(2);
        let result = calculate_gains(holdings, trades.clone(), vec!(), FIAT_CURRENCY.to_string(), Method::FIFO);

        assert!(result.long_term_gain.is_zero());

        let bought_currency_holdings = result.new_holdings.0.get(&trades[0].bought_currency).expect("bought currency not found");
        assert!(!bought_currency_holdings.is_empty());
        assert_eq!(result.short_term_gain, (trades[0].fiat_rate() - currency_holdings[0].rate_in_fiat) * currency_holdings[0].amount + trades[0].fiat_rate() * (trades[0].amount_sold - currency_holdings[0].amount));
    }

    #[test]
    fn single_holding_single_trade_long_term_no_overflow() {
        let holdings = mocks::mock_holdings(1, 1, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = currency_holdings[0].amount;
        let result = calculate_gains(holdings, trades.clone(), vec!(), FIAT_CURRENCY.to_string(), Method::FIFO);

        assert!(result.short_term_gain.is_zero());

        let bought_currency_holdings = result.new_holdings.0.get(&trades[0].bought_currency).expect("bought currency not found");
        assert!(!bought_currency_holdings.is_empty());
        assert_eq!(result.long_term_gain, (trades[0].fiat_rate() - currency_holdings[0].rate_in_fiat) * trades[0].amount_sold);
    }

    #[test]
    fn single_holding_single_trade_long_term_overflow() {
        let holdings = mocks::mock_holdings(1, 1, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = currency_holdings[0].amount * dec!(2);
        let result = calculate_gains(holdings, trades.clone(), vec!(), FIAT_CURRENCY.to_string(), Method::FIFO);

        let bought_currency_holdings = result.new_holdings.0.get(&trades[0].bought_currency).expect("bought currency not found");
        assert!(!bought_currency_holdings.is_empty());

        assert_eq!(result.short_term_gain, trades[0].fiat_rate() * (trades[0].amount_sold - currency_holdings[0].amount));
        assert_eq!(result.long_term_gain, (trades[0].fiat_rate() - currency_holdings[0].rate_in_fiat) * currency_holdings[0].amount);
    }

    #[test]
    fn single_holding_multiple_trade_short_term_no_overflow() {
        let holdings = mocks::mock_holdings(1, 1, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let original_holdings = holdings.clone();
        let currency = original_holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = original_holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(5, mocks::now_u64(), holdings.clone(), false);
        let result = calculate_gains(holdings, trades.clone(), vec!(), FIAT_CURRENCY.to_string(), Method::FIFO);

        assert!(result.long_term_gain.is_zero());

        let mut gain = dec!(0);
        for trade in trades {
            gain += (trade.fiat_rate() - currency_holdings[0].rate_in_fiat) * trade.amount_sold;
        }

        assert_eq!(result.short_term_gain, gain);
    }
}