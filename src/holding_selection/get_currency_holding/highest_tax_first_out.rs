use crate::holding::CurrencyHolding;
use crate::trade::Trade;
use crate::YEAR_IN_MILLISECONDS;

pub fn highest_tax_first_out(trade: Trade, currency_holdings: &[CurrencyHolding]) -> usize {
    currency_holdings
        .iter()
        .enumerate()
        .fold(
            (0, None),
            |(highest_index, unsafe_highest_tax_holding): (usize, Option<&CurrencyHolding>),
             (index, current_currency_holding)| {
                if let Some(highest_tax_holding) = unsafe_highest_tax_holding {
                    if trade.date.wrapping_sub(current_currency_holding.date)
                        >= YEAR_IN_MILLISECONDS
                    {
                        // current holding triggers long term gains
                        if trade.date.wrapping_sub(highest_tax_holding.date) >= YEAR_IN_MILLISECONDS
                            && current_currency_holding.rate_in_fiat
                                < highest_tax_holding.rate_in_fiat
                        {
                            return (index, Some(current_currency_holding));
                        }
                        (highest_index, Some(highest_tax_holding)) // current holding isnt highest
                    } else {
                        // current holding is short term
                        if trade.date.wrapping_sub(highest_tax_holding.date) >= YEAR_IN_MILLISECONDS
                        {
                            // current highest is long term but short term tax is always higher
                            (index, Some(current_currency_holding))
                        } else {
                            if current_currency_holding.rate_in_fiat
                                < highest_tax_holding.rate_in_fiat
                            {
                                return (index, Some(current_currency_holding)); // current holding rate in fiat is lower meaning higher cost
                            }
                            (highest_index, Some(highest_tax_holding))
                        }
                    }
                } else {
                    (index, Some(current_currency_holding))
                }
            },
        )
        .0
}

/*
                if (currentCurrencyHolding.rateInFiat < highestTax.rateInFiat) {
                    highestTaxHoldingIndex = currentIndex;
                    return currentCurrencyHolding;
                }
                return highestTax;


*/
