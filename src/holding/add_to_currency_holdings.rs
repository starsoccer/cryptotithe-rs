use crate::holding::{CurrencyHolding, Holdings};
use rust_decimal::prelude::Decimal;

impl Holdings {
    pub fn add_to_currency_holdings(
        mut self: Holdings,
        currency: String,
        amount: Decimal,
        fiat_rate: Decimal,
        date: u64,
        location: Option<String>,
    ) -> Holdings {
        let currency_holding = CurrencyHolding {
            amount,
            rate_in_fiat: fiat_rate,
            date,
            location: location.unwrap_or_else(|| "".to_owned()),
        };

        if let Some(currency_holdings) = self.0.get_mut(&currency) {
            currency_holdings.push(currency_holding);
        } else {
            self.0.insert(currency, vec![currency_holding]);
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use crate::mocks;
    use rust_decimal::prelude::Zero;

    #[test]
    fn add_new_curreny_holding() {
        let holdings = mocks::mock_holdings(0, 0, None, None);
        assert_eq!(holdings.0.keys().len(), 0);

        let currency = "BTC";
        let new_holdings = holdings.add_to_currency_holdings(
            currency.to_string(),
            Zero::zero(),
            Zero::zero(),
            1234,
            None,
        );

        assert_eq!(new_holdings.0.keys().len(), 1);
    }

    #[test]
    fn add_to_existing_curreny_holding() {
        let holdings = mocks::mock_holdings(1, 3, None, None);
        let holdings_currencies = holdings.0.keys().collect::<Vec<&String>>();
        let currency = holdings_currencies[0].clone();
        assert_eq!(holdings_currencies.len(), 1);

        let new_holdings = holdings.add_to_currency_holdings(
            currency.to_string(),
            Zero::zero(),
            Zero::zero(),
            1234,
            None,
        );

        assert_eq!(new_holdings.0.keys().len(), 1);
        let currency_holding = new_holdings
            .0
            .get(&currency)
            .expect("Unable to get currency holding");
        assert_eq!(currency_holding.len(), 4);
    }
}
