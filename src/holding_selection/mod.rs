use wasm_bindgen::prelude::*;
use crate::{holding, method, trade};
use rust_decimal::prelude::{
    Decimal,
    Zero,
};
use std::clone::Clone;
use rust_decimal_macros::*;
mod get_currency_holding;

#[wasm_bindgen]
pub struct HoldingSelection {
    deducted_holdings: Vec<holding::CurrencyHolding>,
    new_holdings: holding::Holdings,
}

#[wasm_bindgen]
pub fn holding_selection_wasm (
    holdings: &JsValue,
    trade: &JsValue,
    fiat_currency: String,
    method: method::Method,
) -> HoldingSelection {
    let holdings: holding::Holdings = holdings.into_serde().unwrap();
    let trade: trade::Trade = trade.into_serde().unwrap();
    holding_selection(holdings, trade, fiat_currency, method)
}

pub fn holding_selection (
    mut holdings: holding::Holdings,
    trade: trade::Trade,
    fiat_currency: String,
    method: method::Method,
) -> HoldingSelection {
    let mut currency_holding: Vec<holding::CurrencyHolding> = vec!();
    let mut amount_used = trade.amount_sold;

    while !amount_used.is_zero() {
        if let Some(current_currency_holding) = holdings.0.get_mut(&trade.sold_currency) {
            let selected_currency_holding_index = get_currency_holding::get_currency_holding(
                current_currency_holding,
                method,
                trade.clone(),
            );
            let mut selected_currency_holding = current_currency_holding.get_mut(selected_currency_holding_index).unwrap();

            let result = check_currency_holding_amount(amount_used, selected_currency_holding.clone());
            currency_holding.push(result.deducted_currency_holding);
            
            if result.amount_remaining.is_zero() {
                selected_currency_holding.amount = selected_currency_holding.amount - amount_used;
            } else {
                current_currency_holding.remove(selected_currency_holding_index);
            }
            
            amount_used = result.amount_remaining;
        
            if currency_holding.len() == 0 {
                holdings.0.remove(&trade.sold_currency);
            }
        } else {
            if trade.sold_currency == fiat_currency {
                currency_holding.push(holding::CurrencyHolding {
                    amount: amount_used,
                    date: trade.date,
                    rate_in_fiat: dec!(1),
                    location: trade.exchange.clone(),
                });
            } else {
                currency_holding.push(holding::CurrencyHolding {
                    amount: amount_used,
                    date: trade.date,
                    rate_in_fiat: dec!(0),
                    location: trade.exchange.clone(),
                });
            }
            amount_used = Zero::zero()
        }
    }


    return HoldingSelection {
        deducted_holdings: currency_holding,
        new_holdings: holdings,
    };
}

struct CheckCurrencyHoldingAmount {
    pub amount_remaining: Decimal,
    pub deducted_currency_holding: holding::CurrencyHolding,
}

fn check_currency_holding_amount (
    amount_used: Decimal,
    holding_to_check: holding::CurrencyHolding,
) -> CheckCurrencyHoldingAmount {
    if holding_to_check.amount > amount_used {
        return CheckCurrencyHoldingAmount {
            amount_remaining: Zero::zero(),
            deducted_currency_holding: holding::CurrencyHolding {
                amount: amount_used,
                rate_in_fiat: holding_to_check.rate_in_fiat,
                date: holding_to_check.date,
                location: holding_to_check.location,
            },
        };
    } else {
        return CheckCurrencyHoldingAmount {
            amount_remaining: amount_used - holding_to_check.amount,
            deducted_currency_holding: holding::CurrencyHolding {
                amount: holding_to_check.amount,
                rate_in_fiat: holding_to_check.rate_in_fiat,
                date: holding_to_check.date,
                location: holding_to_check.location,
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{holding, method, holding_selection};
    use crate::mocks;
    use rust_decimal_macros::*;
    use rust_decimal::prelude::{
        Decimal,
        Zero,
    };

    fn calculate_total_amount (currency_holdings: Vec<holding::CurrencyHolding>) -> Decimal {
        currency_holdings.into_iter().fold(Zero::zero(), |acc, item| acc + item.amount)
    }
    static FIAT_CURRENCY: &str = "FAKE";

    #[test]
    fn single_holding() {
        let holdings = mocks::mock_holdings(1, 3, None, None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let holdings_total = calculate_total_amount(holdings.0.get(currency).unwrap().clone());
        let mut trades = mocks::mock_trades(
            1,
            123456768,
            holdings.clone(),
            false
        );
        trades[0].amount_sold = holdings.0.get(currency).unwrap()[0].amount;
        trades[0].bought_currency = FIAT_CURRENCY.to_owned().clone();

        let result = holding_selection::holding_selection(holdings.clone(), trades[0].clone(), FIAT_CURRENCY.to_owned().clone(), method::Method::FIFO);

        assert_eq!(calculate_total_amount(result.deducted_holdings), trades[0].amount_sold);

        let currency_holding = result.new_holdings.0.get(currency).expect("cant get new holding currency");
        let t = holdings_total - calculate_total_amount(currency_holding.clone());
        assert_eq!(t, trades[0].amount_sold);
    }

    #[test]
    fn multiple_holding() {
        let holdings = mocks::mock_holdings(1, 3, None, None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let holdings_total = calculate_total_amount(holdings.0.get(currency).unwrap().clone());
        let mut trades = mocks::mock_trades(
            1,
            123456768,
            holdings.clone(),
            false
        );

        trades[0].amount_sold = holdings.0.get(currency).unwrap()[0].amount + dec!(0.01);
        trades[0].bought_currency = FIAT_CURRENCY.to_owned().clone();

        let result = holding_selection::holding_selection(holdings.clone(), trades[0].clone(), FIAT_CURRENCY.to_owned().clone(), method::Method::FIFO);

        assert_eq!(calculate_total_amount(result.deducted_holdings), trades[0].amount_sold);

        let currency_holding = result.new_holdings.0.get(currency).expect("cant get new holding currency");
        let t = holdings_total - calculate_total_amount(currency_holding.clone());
        assert_eq!(t, trades[0].amount_sold);
    }
}