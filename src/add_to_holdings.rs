use crate::holding::{CurrencyHolding, Holdings};
use rust_decimal::prelude::Decimal;

pub fn add_to_holdings(
    old_holdings: Holdings,
    currency: String,
    amount: Decimal,
    fiat_rate: Decimal,
    date: u64,
    location: Option<String>,
) -> Holdings {
    let mut holdings = old_holdings;

    let currency_holding = CurrencyHolding {
        amount,
        rate_in_fiat: fiat_rate,
        date,
        location: location.unwrap_or_else(|| "".to_owned()),
    };

    if let Some(currency_holdings) = holdings.0.get_mut(&currency) {
        currency_holdings.push(currency_holding);
    } else {
        holdings.0.insert(currency, vec![currency_holding]);
    }

    holdings
}

#[cfg(test)]
mod tests {
    use crate::add_to_holdings::add_to_holdings;
    use crate::mocks;
    use rust_decimal_macros::*;

    #[test]
    fn add_new_curreny_holding() {
        let holdings = mocks::mock_holdings(0, 0, None, None);
        assert_eq!(holdings.0.keys().len(), 0);

        let currency = "BTC";
        let new_holdings =
            add_to_holdings(holdings, currency.to_string(), dec!(0), dec!(0), 1234, None);

        assert_eq!(new_holdings.0.keys().len(), 1);
    }

    #[test]
    fn add_to_existing_curreny_holding() {
        let holdings = mocks::mock_holdings(1, 3, None, None);
        let holdings_currencies = &holdings.0.keys();
        let currency = holdings_currencies.clone().collect::<Vec<&String>>()[0];
        assert_eq!(holdings_currencies.len(), 1);

        let new_holdings = add_to_holdings(
            holdings.clone(),
            currency.to_string(),
            dec!(0),
            dec!(0),
            1234,
            None,
        );

        assert_eq!(new_holdings.0.keys().len(), 1);
        let currency_holding = new_holdings
            .0
            .get(currency)
            .expect("Unable to get currency holding");
        assert_eq!(currency_holding.len(), 4);
    }
}
