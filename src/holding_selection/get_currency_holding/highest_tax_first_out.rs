use crate::trade::Trade;
use crate::holding::CurrencyHolding;
use crate::YEAR_IN_MILLISECONDS;

pub fn highest_tax_first_out (trade: Trade, currency_holdings: &Vec<CurrencyHolding>) -> usize {
    currency_holdings.iter().enumerate().fold(
        (0, None), |(highest_index, unsafe_highest_tax_holding): (usize, Option<&CurrencyHolding>), (index, current_currency_holding)| {
            if let Some(highest_tax_holding) = unsafe_highest_tax_holding {
                if trade.date.wrapping_sub(current_currency_holding.date) >= YEAR_IN_MILLISECONDS {
                    if current_currency_holding.rate_in_fiat < highest_tax_holding.rate_in_fiat {
                        return (index, Some(current_currency_holding));
                    }
                    return (highest_index, Some(highest_tax_holding));
                } else if trade.date.wrapping_sub(highest_tax_holding.date) >= YEAR_IN_MILLISECONDS {
                    return (index, Some(current_currency_holding));
                } else {
                    return (highest_index, Some(highest_tax_holding));
                }
            } else {
                return (index, Some(current_currency_holding));
            }
        }
    ).0
}