use crate::holding::{Holdings};
use crate::holding_selection::holding_selection;
use crate::method::Method;
use crate::trade::Trade;
use crate::{MIN_HOLDING_SIZE, YEAR_IN_MILLISECONDS};
use rust_decimal::prelude::{Decimal, Zero};
use rust_decimal_macros::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessedTradeResult {
    pub holdings: Holdings,
    pub cost_basis_trades: Vec<Trade>,
    pub short_term_gain: Decimal,
    pub long_term_gain: Decimal,
    pub short_term_cost_basis: Decimal,
    pub long_term_cost_basis: Decimal,
    pub short_term_proceeds: Decimal,
    pub long_term_proceeds: Decimal,
}

pub fn process_trade(
    original_holdings: Holdings,
    trade: Trade,
    fiat_currency: String,
    method: Method,
) -> ProcessedTradeResult {
    let mut short_term_gain = dec!(0);
    let mut short_term_proceeds = dec!(0);
    let mut short_term_cost_basis = dec!(0);
    let mut long_term_gain = dec!(0);
    let mut long_term_proceeds = dec!(0);
    let mut long_term_cost_basis = dec!(0);

    let mut trades_with_cost_basis: Vec<Trade> = vec![];
    let mut holdings = original_holdings;

    let result = holding_selection(holdings, trade.clone(), fiat_currency.clone(), method);
    holdings = result.new_holdings;

    if trade.sold_currency == fiat_currency {
        holdings = holdings.add_to_currency_holdings(
            trade.bought_currency.clone(),
            trade.amount_sold / trade.rate,
            trade.fiat_rate(),
            trade.date,
            Some(trade.exchange),
        );
    } else {
        let mut fee_fiat_cost = dec!(0);
        let mut amount_to_add = trade.amount_sold / trade.rate;

        if !trade.transaction_fee.is_zero() {
            if trade.transaction_fee_currency == trade.bought_currency {
                fee_fiat_cost += trade.transaction_fee * trade.rate * trade.fiat_rate();
                amount_to_add -= trade.transaction_fee;
            } else if trade.transaction_fee_currency == trade.sold_currency {
                fee_fiat_cost += trade.transaction_fee * trade.fiat_rate();
                amount_to_add -= trade.transaction_fee / trade.rate;
            } else if trade.transaction_fee_currency == fiat_currency {
                fee_fiat_cost += trade.transaction_fee;
                amount_to_add -= trade.transaction_fee / trade.fiat_rate();
            }
        }

        if amount_to_add > MIN_HOLDING_SIZE {
            holdings = holdings.add_to_currency_holdings(
                trade.bought_currency.clone(),
                amount_to_add,
                trade.fiat_rate() * trade.rate,
                trade.date,
                Some(trade.exchange.clone()),
            );
        }

        for holding in result.deducted_holdings {
            let mut gain = (trade.fiat_rate() - holding.rate_in_fiat) * holding.amount;

            if !fee_fiat_cost.is_zero() {
                let fee_cost = holding.amount / trade.amount_sold * fee_fiat_cost;
                gain -= fee_cost;
            }

            let mut trade_to_add = Trade {
                amount_sold: holding.amount,
                short_term: Some(dec!(0)),
                long_term: Some(dec!(0)),
                date_acquired: Some(holding.date),
                cost_basis: Some(holding.rate_in_fiat * holding.amount),
                long_term_trade: Some(false),
                ..trade.clone()
            };

            if trade.date.wrapping_sub(holding.date) > YEAR_IN_MILLISECONDS {
                trade_to_add.long_term = Some(gain);
                long_term_gain += gain;
                long_term_proceeds += trade_to_add.fiat_rate() * trade_to_add.amount_sold;
                long_term_cost_basis += trade_to_add.cost_basis();
                trade_to_add.long_term_trade = Some(true);
            } else {
                trade_to_add.short_term = Some(gain);
                short_term_gain += gain;
                short_term_proceeds += trade_to_add.fiat_rate() * trade_to_add.amount_sold;
                short_term_cost_basis += trade_to_add.cost_basis();
            }

            trades_with_cost_basis.push(trade_to_add);
        }
    }

    ProcessedTradeResult {
        holdings,
        cost_basis_trades: trades_with_cost_basis,
        short_term_gain,
        long_term_gain,
        short_term_cost_basis,
        long_term_cost_basis,
        short_term_proceeds,
        long_term_proceeds,
    }
}

#[cfg(test)]
mod tests {
    use super::process_trade;
    use crate::method;
    use crate::mocks;
    use crate::{QUARTER_IN_MILLISECONDS, YEAR_IN_MILLISECONDS, trade::Trade, holding::Holdings};
    use rust_decimal::prelude::{Decimal, Zero};
    use rust_decimal_macros::*;

    static FIAT_CURRENCY: &str = "FAKE";

    struct Info {
        cost_basis: Decimal,
        gain: Decimal,
        proceeds: Decimal,
        deducted_count: usize,
    }

    fn calculate_info (trade: Trade, holdings: Holdings, currency: &String) -> Info {
        let mut cost_basis = dec!(0);
        let mut gain = dec!(0);
        let mut proceeds = dec!(0);
        let mut deducted_count = 0;
        let mut amount_left = trade.amount_sold;

        for currency_holding in holdings
            .0
            .get(currency)
            .expect("unable to get holding by currency")
        {
            deducted_count += 1;
            if amount_left > currency_holding.amount {
                amount_left -= currency_holding.amount;
                cost_basis += currency_holding.rate_in_fiat * currency_holding.amount;
                gain += (trade.fiat_rate() - currency_holding.rate_in_fiat)
                    * currency_holding.amount;
                proceeds += currency_holding.amount * trade.fiat_rate();
                // todo add test with fee
            } else {
                cost_basis += currency_holding.rate_in_fiat * amount_left;
                gain += (trade.fiat_rate() - currency_holding.rate_in_fiat) * amount_left; // todo add test with fee
                proceeds += amount_left * trade.fiat_rate();
                break;
            }
        }

        Info {
            cost_basis,
            gain,
            proceeds,
            deducted_count,
        } 
    }
    
    #[test]
    fn short_term_trade_single_holdings() {
        let holdings = mocks::mock_holdings(1, 10, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = holdings.0.get(currency).unwrap()[0].amount;
        trades[0].bought_currency = FIAT_CURRENCY.to_owned().clone();

        let result = process_trade(
            holdings.clone(),
            trades[0].clone(),
            FIAT_CURRENCY.to_string(),
            method::Method::FIFO,
        );

        let info = calculate_info(trades[0].clone(), holdings.clone(), currency);

        assert_ne!(result.holdings, holdings);
        assert!(result.long_term_proceeds.is_zero());
        assert!(result.long_term_cost_basis.is_zero());
        assert!(result.long_term_gain.is_zero());

        assert_eq!(result.cost_basis_trades.len(), info.deducted_count);
        assert_eq!(result.short_term_gain, info.gain);
        assert_eq!(result.short_term_cost_basis, info.cost_basis);
        assert_eq!(
            result.short_term_proceeds,
            info.proceeds
        );
    }

    #[test]
    fn short_term_trade_multiple_holdings() {
        let holdings = mocks::mock_holdings(1, 10, Some(mocks::now_u64() - QUARTER_IN_MILLISECONDS), None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = holdings.0.get(currency).unwrap()[0].amount * dec!(2);
        trades[0].bought_currency = FIAT_CURRENCY.to_owned().clone();

        let result = process_trade(
            holdings.clone(),
            trades[0].clone(),
            FIAT_CURRENCY.to_string(),
            method::Method::FIFO,
        );

        let info = calculate_info(trades[0].clone(), holdings.clone(), currency);

        assert_ne!(result.holdings, holdings);
        assert!(result.long_term_proceeds.is_zero());
        assert!(result.long_term_cost_basis.is_zero());
        assert!(result.long_term_gain.is_zero());

        assert_eq!(result.cost_basis_trades.len(), info.deducted_count);
        assert_eq!(result.short_term_gain, info.gain);
        assert_eq!(result.short_term_cost_basis, info.cost_basis);
        assert_eq!(result.short_term_proceeds, info.proceeds);
    }

    #[test]
    fn long_term_trade_single_holdings() {
        let holdings = mocks::mock_holdings(1, 10, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = holdings.0.get(currency).unwrap()[0].amount;
        trades[0].bought_currency = FIAT_CURRENCY.to_owned().clone();

        let result = process_trade(
            holdings.clone(),
            trades[0].clone(),
            FIAT_CURRENCY.to_string(),
            method::Method::FIFO,
        );

        let info = calculate_info(trades[0].clone(), holdings.clone(), currency);

        assert_ne!(result.holdings, holdings);
        assert!(result.short_term_gain.is_zero());
        assert!(result.short_term_cost_basis.is_zero());
        assert!(result.short_term_gain.is_zero());

        assert_eq!(result.cost_basis_trades.len(), info.deducted_count);
        assert_eq!(result.long_term_gain, info.gain);
        assert_eq!(result.long_term_cost_basis, info.cost_basis);
        assert_eq!(result.long_term_proceeds, info.proceeds);
    }

    #[test]
    fn long_term_trade_multiple_holdings() {
        let holdings = mocks::mock_holdings(1, 10, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = holdings.0.get(currency).unwrap()[0].amount * dec!(2);
        trades[0].bought_currency = FIAT_CURRENCY.to_owned().clone();

        let result = process_trade(
            holdings.clone(),
            trades[0].clone(),
            FIAT_CURRENCY.to_string(),
            method::Method::FIFO,
        );

        let info = calculate_info(trades[0].clone(), holdings.clone(), currency);

        assert_ne!(result.holdings, holdings);
        assert!(result.short_term_gain.is_zero());
        assert!(result.short_term_cost_basis.is_zero());
        assert!(result.short_term_gain.is_zero());

        assert_eq!(result.cost_basis_trades.len(), info.deducted_count);
        assert_eq!(result.long_term_gain, info.gain);
        assert_eq!(result.long_term_cost_basis, info.cost_basis);
        assert_eq!(result.long_term_proceeds, info.proceeds);
    }

    #[test]
    fn short_long_term_trade_multiple_holdings() {
        let mut holdings = mocks::mock_holdings(1, 10, None, Some(mocks::now_u64() - YEAR_IN_MILLISECONDS));
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0].clone();
        let currency_holdings = holdings.0.get_mut(&currency).unwrap();
        currency_holdings[0].date = mocks::now_u64() - QUARTER_IN_MILLISECONDS;

        let mut trades = mocks::mock_trades(1, mocks::now_u64(), holdings.clone(), false);
        trades[0].amount_sold = holdings.0.get(&currency).unwrap()[0].amount * dec!(2);
        trades[0].bought_currency = FIAT_CURRENCY.to_owned().clone();

        let result = process_trade(
            holdings.clone(),
            trades[0].clone(),
            FIAT_CURRENCY.to_string(),
            method::Method::FIFO,
        );

        let info = calculate_info(trades[0].clone(), holdings.clone(), &currency);

        assert_ne!(result.holdings, holdings);

        assert!(!result.short_term_cost_basis.is_zero());
        assert!(!result.short_term_gain.is_zero());
        assert!(!result.short_term_proceeds.is_zero());

        assert!(!result.long_term_cost_basis.is_zero());
        assert!(!result.long_term_gain.is_zero());
        assert!(!result.long_term_proceeds.is_zero());

        assert_eq!(result.cost_basis_trades.len(), info.deducted_count);
        assert_eq!(result.long_term_gain + result.short_term_gain, info.gain);
        assert_eq!(result.long_term_cost_basis + result.short_term_cost_basis, info.cost_basis);
        assert_eq!(result.long_term_proceeds + result.short_term_proceeds, info.proceeds);
    }
}
