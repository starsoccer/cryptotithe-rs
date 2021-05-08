use crate::{holding, method, trade};
use serde::{Deserialize, Serialize};
mod cost_first_out;
mod highest_tax_first_out;
mod lowest_tax_first_out;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetHolding {
    pub holding: holding::CurrencyHolding,
    #[serde(rename = "startingIndex")]
    pub starting_index: u32,
    #[serde(rename = "endingIndex")]
    pub ending_index: u32,
}

pub fn get_currency_holding(
    currency_holdings: &[holding::CurrencyHolding],
    method: method::Method,
    trade: trade::Trade,
) -> usize {
    //holdings.0.get_mut(&trade.sold_currency).unwrap();
    match method {
        method::Method::LTFO => {
            lowest_tax_first_out::lowest_tax_first_out(trade, currency_holdings)
        }
        method::Method::HTFO => {
            highest_tax_first_out::highest_tax_first_out(trade, currency_holdings)
        }
        method::Method::LCFO => cost_first_out::cost_first_out(currency_holdings, false),
        method::Method::HCFO => cost_first_out::cost_first_out(currency_holdings, true),
        method::Method::LIFO => currency_holdings.len() - 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::get_currency_holding;
    use crate::holding::CurrencyHolding;
    use crate::mocks;
    use crate::{method, YEAR_IN_MILLISECONDS};
    use std::time::SystemTime;

    #[test]
    fn get_currency_holding_fifo() {
        let holdings = mocks::mock_holdings(1, 10, None, None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);

        let result =
            get_currency_holding(currency_holdings, method::Method::FIFO, trades[0].clone());

        assert_eq!(result, 0);
        assert_eq!(
            &currency_holdings[result],
            currency_holdings.first().unwrap()
        );
    }

    #[test]
    fn get_currency_holding_lifo() {
        let holdings = mocks::mock_holdings(1, 10, None, None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);

        let result =
            get_currency_holding(currency_holdings, method::Method::LIFO, trades[0].clone());

        assert_eq!(result, currency_holdings.len() - 1);
        assert_eq!(
            &currency_holdings[result],
            currency_holdings.last().unwrap()
        );
    }

    #[test]
    fn get_currency_holding_hcfo() {
        let holdings = mocks::mock_holdings(1, 10, None, None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);

        let result =
            get_currency_holding(currency_holdings, method::Method::HCFO, trades[0].clone());

        let mut highest_cost_holding_index = 0;
        for (index, current_currency_holding) in currency_holdings.iter().enumerate() {
            if current_currency_holding.rate_in_fiat
                > currency_holdings[highest_cost_holding_index].rate_in_fiat
            {
                highest_cost_holding_index = index;
            }
        }

        assert_eq!(result, highest_cost_holding_index);
        assert_eq!(
            currency_holdings[result],
            currency_holdings[highest_cost_holding_index]
        );
    }

    #[test]
    fn get_currency_holding_lcfo() {
        let holdings = mocks::mock_holdings(1, 10, None, None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = holdings.0.get(currency).unwrap();
        let trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);

        let result =
            get_currency_holding(currency_holdings, method::Method::LCFO, trades[0].clone());

        let mut lowest_cost_holding_index = 0;
        for (index, current_currency_holding) in currency_holdings.iter().enumerate() {
            if current_currency_holding.rate_in_fiat
                < currency_holdings[lowest_cost_holding_index].rate_in_fiat
            {
                lowest_cost_holding_index = index;
            }
        }

        assert_eq!(result, lowest_cost_holding_index);
        assert_eq!(
            currency_holdings[result],
            currency_holdings[lowest_cost_holding_index]
        );
    }

    #[test]
    fn get_currency_holding_htfo_short_term() {
        let holdings = mocks::mock_holdings(1, 10, Some(mocks::now_u64() - 30844800000), None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);
        trades[0].sold_currency = currency.into();
        trades[0].date = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis() as u64;

        let htfo_result =
            get_currency_holding(currency_holdings, method::Method::HTFO, trades[0].clone());
        let lcfo_result =
            get_currency_holding(currency_holdings, method::Method::LCFO, trades[0].clone());

        println!("{:?}", currency_holdings);
        println!("{:?}", trades[0]);

        assert_eq!(htfo_result, lcfo_result);
    }

    #[test]
    fn get_currency_holding_htfo_long_term() {
        let holdings = mocks::mock_holdings(
            1,
            10,
            Some(mocks::now_u64() - YEAR_IN_MILLISECONDS * 3),
            Some(mocks::now_u64() - YEAR_IN_MILLISECONDS * 2),
        );
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);
        trades[0].sold_currency = currency.into();
        trades[0].date = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis() as u64;

        let htfo_result =
            get_currency_holding(currency_holdings, method::Method::HTFO, trades[0].clone());
        let lcfo_result =
            get_currency_holding(currency_holdings, method::Method::LCFO, trades[0].clone());

        assert_eq!(htfo_result, lcfo_result);
    }

    #[test]
    fn get_currency_holding_htfo_short_and_long_term() {
        let mut holdings = mocks::mock_holdings(
            1,
            10,
            Some(mocks::now_u64() - YEAR_IN_MILLISECONDS * 3),
            None,
        );
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0].clone();
        holdings.0.get_mut(&currency).unwrap()[0].date = mocks::now_u64();
        let currency_holdings = holdings.0.get(&currency).unwrap();

        let mut conditions_met = (false, false);
        for holding in currency_holdings {
            if mocks::now_u64().wrapping_sub(holding.date) > YEAR_IN_MILLISECONDS {
                conditions_met.1 = true;
            } else {
                conditions_met.0 = true;
            }
        }

        // ensure both a long term and short term holdings exist
        assert_eq!(conditions_met, (true, true));

        let mut trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);
        trades[0].sold_currency = currency.into();
        trades[0].date = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis() as u64;

        let htfo_result =
            get_currency_holding(currency_holdings, method::Method::HTFO, trades[0].clone());

        let lcfo_currency_holdings = {
            let short_term_currency_holdings: Vec<CurrencyHolding> = currency_holdings
                .clone()
                .into_iter()
                .filter(|c| trades[0].date.wrapping_sub(c.date) <= YEAR_IN_MILLISECONDS)
                .collect();
            if short_term_currency_holdings.len() > 0 {
                short_term_currency_holdings
            } else {
                currency_holdings.clone()
            }
        };
        let lcfo_result = get_currency_holding(
            &lcfo_currency_holdings,
            method::Method::LCFO,
            trades[0].clone(),
        );

        assert_eq!(
            currency_holdings[htfo_result],
            lcfo_currency_holdings[lcfo_result]
        );
    }

    #[test]
    fn get_currency_holding_ltfo_short_term() {
        let holdings = mocks::mock_holdings(1, 10, Some(mocks::now_u64() - 30844800000), None);
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);
        trades[0].sold_currency = currency.into();
        trades[0].date = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis() as u64;

        let ltfo_result =
            get_currency_holding(currency_holdings, method::Method::LTFO, trades[0].clone());
        let hcfo_result =
            get_currency_holding(currency_holdings, method::Method::HCFO, trades[0].clone());

        assert_eq!(ltfo_result, hcfo_result);
    }

    #[test]
    fn get_currency_holding_ltfo_long_term() {
        let holdings = mocks::mock_holdings(
            1,
            10,
            Some(mocks::now_u64() - YEAR_IN_MILLISECONDS * 3),
            Some(mocks::now_u64() - YEAR_IN_MILLISECONDS * 2),
        );
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0];
        let currency_holdings = holdings.0.get(currency).unwrap();
        let mut trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);
        trades[0].sold_currency = currency.into();
        trades[0].date = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis() as u64;

        let ltfo_result =
            get_currency_holding(currency_holdings, method::Method::LTFO, trades[0].clone());
        let hcfo_result =
            get_currency_holding(currency_holdings, method::Method::HCFO, trades[0].clone());

        assert_eq!(ltfo_result, hcfo_result);
    }

    #[test]
    fn get_currency_holding_ltfo_short_and_long_term() {
        let mut holdings = mocks::mock_holdings(
            1,
            10,
            Some(mocks::now_u64() - YEAR_IN_MILLISECONDS * 3),
            None,
        );
        let currency = holdings.0.keys().collect::<Vec<&String>>()[0].clone();
        holdings.0.get_mut(&currency).unwrap()[0].date = mocks::now_u64();
        let currency_holdings = holdings.0.get(&currency).unwrap();

        let mut conditions_met = (false, false);
        for holding in currency_holdings {
            if mocks::now_u64().wrapping_sub(holding.date) > YEAR_IN_MILLISECONDS {
                conditions_met.1 = true;
            } else {
                conditions_met.0 = true;
            }
        }

        // ensure both a long term and short term holdings exist
        assert_eq!(conditions_met, (true, true));

        let mut trades = mocks::mock_trades(1, 123456768, holdings.clone(), false);
        trades[0].sold_currency = currency.into();
        trades[0].date = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis() as u64;

        let ltfo_result =
            get_currency_holding(currency_holdings, method::Method::LTFO, trades[0].clone());

        let hcfo_currency_holdings = {
            let long_term_currency_holdings: Vec<CurrencyHolding> = currency_holdings
                .clone()
                .into_iter()
                .filter(|c| trades[0].date.wrapping_sub(c.date) >= YEAR_IN_MILLISECONDS)
                .collect();
            if long_term_currency_holdings.len() > 0 {
                long_term_currency_holdings
            } else {
                currency_holdings.clone()
            }
        };
        let hcfo_result = get_currency_holding(
            &hcfo_currency_holdings,
            method::Method::HCFO,
            trades[0].clone(),
        );

        assert_eq!(
            currency_holdings[ltfo_result],
            hcfo_currency_holdings[hcfo_result]
        );
    }
}
