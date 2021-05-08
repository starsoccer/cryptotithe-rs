use crate::holding::CurrencyHolding;

pub fn cost_first_out(currency_holdings: &Vec<CurrencyHolding>, highest: bool) -> usize {
    currency_holdings
        .iter()
        .enumerate()
        .fold((0, None), if highest { highest_fold } else { lowest_fold })
        .0
}

type FoldOptionType<'a> = (usize, Option<&'a CurrencyHolding>);
type FoldType<'a> = (usize, &'a CurrencyHolding);

fn highest_fold<'a>(acc: FoldOptionType<'a>, current: FoldType<'a>) -> FoldOptionType<'a> {
    fold(true, acc, current)
}

fn lowest_fold<'a>(acc: FoldOptionType<'a>, current: FoldType<'a>) -> FoldOptionType<'a> {
    fold(false, acc, current)
}

fn fold<'a>(
    highest: bool,
    (highest_index, highest_holding): FoldOptionType<'a>,
    (current_index, current_holding): FoldType<'a>,
) -> FoldOptionType<'a> {
    if let Some(safe_highest_holding) = highest_holding {
        let result = if highest {
            safe_highest_holding.rate_in_fiat > current_holding.rate_in_fiat
        } else {
            safe_highest_holding.rate_in_fiat < current_holding.rate_in_fiat
        };

        match result {
            true => (highest_index, Some(&safe_highest_holding)),
            false => (current_index, Some(current_holding)),
        }
    } else {
        (current_index, Some(current_holding))
    }
}
