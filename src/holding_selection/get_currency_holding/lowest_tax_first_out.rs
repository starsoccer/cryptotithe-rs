use crate::holding::CurrencyHolding;
use crate::trade::Trade;
use crate::YEAR_IN_MILLISECONDS;

pub fn lowest_tax_first_out(trade: Trade, currency_holdings: &Vec<CurrencyHolding>) -> usize {
    currency_holdings
        .iter()
        .enumerate()
        .fold(
            (0, None),
            |(highest_index, unsafe_lowest_tax_holding): (usize, Option<&CurrencyHolding>),
             (index, current_currency_holding)| {
                if let Some(lowest_tax_holding) = unsafe_lowest_tax_holding {
                    if trade.date.wrapping_sub(current_currency_holding.date)
                        >= YEAR_IN_MILLISECONDS
                    {
                        if trade.date.wrapping_sub(lowest_tax_holding.date) >= YEAR_IN_MILLISECONDS
                        {
                            if current_currency_holding.rate_in_fiat
                                > lowest_tax_holding.rate_in_fiat
                            {
                                return (index, Some(current_currency_holding));
                            }
                            (highest_index, Some(lowest_tax_holding))
                        } else {
                            (index, Some(current_currency_holding))
                        }
                    } else {
                        if trade.date.wrapping_sub(lowest_tax_holding.date) < YEAR_IN_MILLISECONDS
                            && current_currency_holding.rate_in_fiat
                                > lowest_tax_holding.rate_in_fiat
                        {
                            return (index, Some(current_currency_holding));
                        }
                        (highest_index, Some(lowest_tax_holding))
                    }
                } else {
                    (index, Some(current_currency_holding))
                }
            },
        )
        .0
}
