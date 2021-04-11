use crate::holding::CurrencyHolding;
use crate::trade::Trade;

pub fn tax_first_out (currency_holdings: &Vec<CurrencyHolding>, trade: Trade, highest: bool) -> usize {
    currency_holdings.iter().enumerate().fold(
        (0, None),
        |acc,
        x| if highest {
            highest_fold(trade, acc, x)
        } else {
            highest_fold(trade, acc, x)
        }
    ).0
}

type FoldOptionType<'a> = (usize, Option<&'a CurrencyHolding>);
type FoldType<'a> = (usize, &'a CurrencyHolding);

fn highest_fold<'a> (trade: Trade, (highest_index, highest_holding): FoldOptionType<'a>, (current_index, current_holding): FoldType<'a> -> FoldOptionType<'a> {
    if let Some(safe_highest_holding) = highest_holding {
        if trade.date - current_holding.date >= YEAR_IN_MILLISECONDS {
            // is long term gains
            if trade.date - highest_holding.date >= YEAR_IN_MILLISECONDS && current_holding.rate_in_fiat < highest_holding.rate_in_fiat {
                (current_index, Some(current_holding))
            } else {
                (highest_index, Some(&safe_highest_holding))
            }
        } else {
            // is short term gains
            if trade.date - highest_holding.date >= YEAR_IN_MILLISECONDS || current_holding.rate_in_fiat < highest_holding.rate_in_fiat {
                // if highest is current long term gains OR current rate in fiat is less then highest
                (current_index, Some(current_holding))
            } else {
                (highest_index, Some(&safe_highest_holding))
            }
        }
    } else {
        (current_index, Some(current_holding))
    }
}