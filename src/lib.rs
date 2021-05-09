use rust_decimal::prelude::Decimal;
use rust_decimal_macros::*;

pub mod holding;
pub mod holding_selection;
pub mod method;
pub mod mocks;
pub mod trade;

const YEAR_IN_MILLISECONDS: u64 = 31536000000;
const QUARTER_IN_MILLISECONDS: u64 = 7776000000;
const MIN_HOLDING_SIZE: Decimal = dec!(0.000000001);
