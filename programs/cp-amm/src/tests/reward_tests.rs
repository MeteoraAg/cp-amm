use std::{u128, u64};

use crate::{
    constants::{LIQUIDITY_MAX, MAX_SQRT_PRICE, MIN_SQRT_PRICE},
    curve::get_initialize_amounts,
    params::swap::TradeDirection,
    safe_math::SafeMath,
    state::Pool,
};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000, .. ProptestConfig::default()
    })]
    #[test]
    fn test_reserve_wont_lost_when_swap_from_a_to_b(
        sqrt_price in MIN_SQRT_PRICE..=MAX_SQRT_PRICE,
        amount_in in 1..=u64::MAX,
        liquidity in 1..=LIQUIDITY_MAX,
    ) {
        let mut pool = Pool {
            liquidity,
            sqrt_price,
            sqrt_min_price: MIN_SQRT_PRICE,
            sqrt_max_price: MAX_SQRT_PRICE,
            ..Default::default()
        };

        let trade_direction = TradeDirection::AtoB;

        let max_amount_in = pool.get_max_amount_in(trade_direction).unwrap();
        if amount_in <= max_amount_in {
            let swap_result_0 = pool
            .get_swap_result(amount_in, false, trade_direction)
            .unwrap();

            pool.apply_swap_result(&swap_result_0, trade_direction, 0).unwrap();
            // swap back

            let swap_result_1 = pool
            .get_swap_result(swap_result_0.output_amount, false, TradeDirection::BtoA)
            .unwrap();

            assert!(swap_result_1.output_amount < amount_in);
        }

    }


    #[test]
    fn test_reserve_wont_lost_when_swap_from_b_to_a(
        sqrt_price in MIN_SQRT_PRICE..=MAX_SQRT_PRICE,
        amount_in in 1..=u64::MAX,
        liquidity in 1..=LIQUIDITY_MAX,
    ) {
        let mut pool = Pool {
            liquidity,
            sqrt_price,
            sqrt_min_price: MIN_SQRT_PRICE,
            sqrt_max_price: MAX_SQRT_PRICE,
            ..Default::default()
        };

        let trade_direction = TradeDirection::BtoA;

        let max_amount_in = pool.get_max_amount_in(trade_direction).unwrap();
        if amount_in <= max_amount_in {
            let swap_result_0 = pool
            .get_swap_result(amount_in, false, trade_direction)
            .unwrap();

            pool.apply_swap_result(&swap_result_0, trade_direction, 0).unwrap();
            // swap back

            let swap_result_1 = pool
            .get_swap_result(swap_result_0.output_amount, false, TradeDirection::AtoB)
            .unwrap();

            assert!(swap_result_1.output_amount < amount_in);
        }
    }

}

#[test]
fn test_swap_basic() {
    // let pool_fees = PoolFeesStruct {
    //     trade_fee_numerator: 1_000_000, //1%
    //     protocol_fee_percent: 20,
    //     partner_fee_percent: 50,
    //     referral_fee_percent: 20,
    //     ..Default::default()
    // };
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = u64::MAX as u128;
    let liquidity = LIQUIDITY_MAX;
    let mut pool = Pool {
        // pool_fees,
        ..Default::default()
    };

    let (_token_a_amount, _token_b_amount) =
        get_initialize_amounts(sqrt_min_price, sqrt_max_price, sqrt_price, liquidity).unwrap();
    // println!("amount {} {}", _token_a_amount, _token_b_amount);
    pool.liquidity = liquidity;
    pool.sqrt_max_price = sqrt_max_price;
    pool.sqrt_min_price = sqrt_min_price;
    pool.sqrt_price = sqrt_price;

    // let next_sqrt_price =
    //     get_next_sqrt_price_from_input(sqrt_price, liquidity, 100_000_000, true).unwrap();

    // println!(
    //     "price {} {} {}",
    //     to_decimal(sqrt_price),
    //     to_decimal(next_sqrt_price),
    //     liquidity.safe_shr(64).unwrap(),
    // );

    let amount_in = 100_000_000;

    let is_referral = false;
    let trade_direction = TradeDirection::AtoB;
    let swap_result = pool
        .get_swap_result(amount_in, is_referral, trade_direction)
        .unwrap();

    println!("result {:?}", swap_result);

    // return;

    pool.apply_swap_result(&swap_result, trade_direction, 0)
        .unwrap();

    let swap_result_referse = pool
        .get_swap_result(swap_result.output_amount, is_referral, TradeDirection::BtoA)
        .unwrap();

    println!("reverse {:?}", swap_result_referse);
    assert!(swap_result_referse.output_amount <= amount_in);
}

#[test]
fn test_basic_math() {
    let liquidity = LIQUIDITY_MAX;
    let quote_1 = liquidity.safe_shr(64).unwrap();
    let quote_2 = liquidity.safe_div(1.safe_shl(64).unwrap()).unwrap();
    assert_eq!(quote_1, quote_2);
}
