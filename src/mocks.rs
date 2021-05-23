use crate::{holding, trade, QUARTER_IN_MILLISECONDS};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rust_decimal::prelude::{Decimal, FromPrimitive, ToPrimitive, Zero};
use rust_decimal_macros::*;
use std::collections::HashMap;
use std::time::SystemTime;

fn rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

fn rand_decimal() -> Decimal {
    Decimal::from_f64(thread_rng().gen()).unwrap()
}

pub fn now_u64() -> u64 {
    SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis() as u64
}

const DEFAULT_STARTING: u64 = 1262322000000;

fn date_in_range(start: Option<u64>, end: Option<u64>) -> u64 {
    let starting_date = start.unwrap_or(DEFAULT_STARTING);
    let ending_date = end.unwrap_or_else(now_u64);

    thread_rng().gen_range(starting_date..ending_date)
}

pub fn mock_currency_holdings(
    amount: u32,
    starting_date: Option<u64>,
    ending_date: Option<u64>,
) -> Vec<holding::CurrencyHolding> {
    let mut currency_holdings: Vec<holding::CurrencyHolding> = vec![];
    for _ in 0..amount {
        currency_holdings.push(holding::CurrencyHolding {
            amount: rand_decimal(),
            rate_in_fiat: rand_decimal(),
            date: date_in_range(starting_date, ending_date),
            location: rand_string(),
        });
    }

    currency_holdings
}

pub fn mock_holdings(
    currencies: u32,
    holdings_per_currency: u32,
    starting_date: Option<u64>,
    ending_date: Option<u64>,
) -> holding::Holdings {
    let mut holdings = holding::Holdings(HashMap::new());

    for _ in 0..currencies {
        holdings.0.insert(
            rand_string(),
            mock_currency_holdings(holdings_per_currency, starting_date, ending_date),
        );
    }

    holdings
}

pub fn mock_trades(
    amount: u32,
    starting_date: u64,
    current_holdings: holding::Holdings,
    allow_overflow: bool,
) -> Vec<trade::Trade> {
    let mut trades: Vec<trade::Trade> = vec![];
    let currencies = {
        let keys = current_holdings.0.keys().cloned().collect::<Vec<String>>();
        if keys.is_empty() {
            vec![rand_string()]
        } else {
            keys
        }
    };

    for currency in currencies {
        let a = Vec::new();
        let currency_holdings = current_holdings.0.get(&currency).unwrap_or(&a);
        let total_holdings: Decimal = currency_holdings
            .iter()
            .fold(Zero::zero(), |acc, item| acc + item.amount);

        for _ in 0..amount {
            let bought_currency: String = rand_string();
            let amount_sold = if allow_overflow {
                total_holdings + rand_decimal()
            } else {
                let max_per_trade = (total_holdings / Decimal::from_u32(amount).unwrap()
                    * dec!(100))
                .to_u32()
                .unwrap();

                let rand: u32 = thread_rng().gen_range(0..(max_per_trade + 1));
                Decimal::from_u32(rand).unwrap() / dec!(100)
            };

            trades.push(trade::Trade {
                bought_currency,
                sold_currency: currency.clone(),
                amount_sold,
                rate: rand_decimal(),
                date: date_in_range(
                    Some(starting_date),
                    Some(now_u64() + QUARTER_IN_MILLISECONDS),
                ),
                exchange_id: rand_string(),
                exchange: rand_string(),
                id: rand_string(),
                transaction_fee: Zero::zero(),
                transaction_fee_currency: rand_string(),
                fiat_rate: Some(rand_decimal()),
                short_term: None,
                long_term: None,
                date_acquired: None,
                cost_basis: None,
                long_term_trade: None,
            });
        }
    }

    trades
}
