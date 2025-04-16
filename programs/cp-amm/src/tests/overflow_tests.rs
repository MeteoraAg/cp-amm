use std::u128;

use proptest::proptest;
use ruint::aliases::U256;

use crate::{
    constants::{MAX_SQRT_PRICE, MIN_SQRT_PRICE},
    curve::{get_initialize_amounts, get_next_sqrt_price, RESOLUTION},
    tests::LIQUIDITY_MAX,
};
use proptest::prelude::*;

proptest! {
#![proptest_config(ProptestConfig {
    cases: 10000, .. ProptestConfig::default()
})]
#[test]
fn test_get_initialize_amounts(
    sqrt_price in MIN_SQRT_PRICE..=MAX_SQRT_PRICE,
    liquidity in 1..=LIQUIDITY_MAX,
    ) {
        let sqrt_min_price = MIN_SQRT_PRICE;
        let sqrt_max_price = MAX_SQRT_PRICE;
        get_initialize_amounts(sqrt_min_price, sqrt_max_price, sqrt_price, liquidity).unwrap();
    }

#[test]
fn test_get_next_sqrt_price_from_input_a_for_b(
    sqrt_price in MIN_SQRT_PRICE..=MAX_SQRT_PRICE,
    liquidity in 1..=LIQUIDITY_MAX,
    amount_in in 0..=u64::MAX,
    ) {
        get_next_sqrt_price(sqrt_price, liquidity, amount_in, true, false).unwrap();
    }
#[test]
fn test_get_next_sqrt_price_from_input_b_for_a(
    sqrt_price in MIN_SQRT_PRICE..=MAX_SQRT_PRICE,
    liquidity in 1..=LIQUIDITY_MAX,
    amount_in in 0..=u64::MAX,
    ) {
        get_next_sqrt_price(sqrt_price, liquidity, amount_in, false, false).unwrap();
    }
}

#[test]
fn test_get_initialize_amounts_single_case() {
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = 29079168020;
    let liquidity = 13729854716085099837338887321;
    let (a, b) =
        get_initialize_amounts(sqrt_min_price, sqrt_max_price, sqrt_price, liquidity).unwrap();
    println!("{} {}", a, b);
}

#[test]
fn get_next_sqrt_price_from_input_single_test() {
    let sqrt_price = MAX_SQRT_PRICE / 2;
    let liquidity = 1372985471608509983;
    let amount_in = 100_000_00;
    let next_price = get_next_sqrt_price(sqrt_price, liquidity, amount_in, true, false).unwrap();
    println!("{}", next_price);

    let next_price = get_next_sqrt_price(sqrt_price, liquidity, amount_in, false, false).unwrap();
    println!("{}", next_price);
}

// Helper function to convert fixed point number to decimal with 10^12 precision. Decimal form is not being used in program, it's only for UI purpose.
pub fn _to_decimal(sqrt_price: u128) -> u128 {
    let value = U256::from(sqrt_price) * U256::from(sqrt_price);
    let precision = U256::from(1_000_000u128);
    let scaled_value = value.checked_mul(precision).unwrap();
    // ruint checked math is different with the rust std u128. If there's bit with 1 value being shifted out, it will return None. Therefore, we use overflowing_shr
    let (scaled_down_value, _) = scaled_value.overflowing_shr(2 * RESOLUTION as usize);
    scaled_down_value.try_into().unwrap()
}
