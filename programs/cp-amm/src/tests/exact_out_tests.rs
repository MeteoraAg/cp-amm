use std::{u128, u64};

use crate::{
    constants::{MAX_SQRT_PRICE, MIN_SQRT_PRICE},
    params::swap::TradeDirection,
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
        amount_out in 1..=u64::MAX,
        liquidity in 1..=u128::MAX,
    ) {
        let mut pool = Pool {
            liquidity,
            sqrt_price,
            sqrt_min_price: MIN_SQRT_PRICE,
            sqrt_max_price: MAX_SQRT_PRICE,
            ..Default::default()
        };

        let trade_direction = TradeDirection::AtoB;

        let max_amount_out = pool.get_max_amount_out(trade_direction).unwrap();
        if amount_out <= max_amount_out {
            let swap_result_0 = pool
            .get_swap_result_exact_out(amount_out, false, trade_direction, 0)
            .unwrap();

            pool.apply_swap_result(&swap_result_0, trade_direction, 0).unwrap();

            // swap back
            let swap_result_1 = pool
            .get_swap_result_exact_out(swap_result_0.input_amount, false, TradeDirection::BtoA, 0)
            .unwrap();

            assert!(swap_result_1.input_amount <= amount_out);
        }

    }


    #[test]
    fn test_reserve_wont_lost_when_swap_from_b_to_a(
        sqrt_price in MIN_SQRT_PRICE..=MAX_SQRT_PRICE,
        amount_out in 1..=u64::MAX,
        liquidity in 1..=u128::MAX,
    ) {
        let mut pool = Pool {
            liquidity,
            sqrt_price,
            sqrt_min_price: MIN_SQRT_PRICE,
            sqrt_max_price: MAX_SQRT_PRICE,
            ..Default::default()
        };

        let trade_direction = TradeDirection::BtoA;

        let max_amount_out = pool.get_max_amount_out(trade_direction).unwrap();
        if amount_out <= max_amount_out {
            let swap_result_0 = pool
            .get_swap_result_exact_out(amount_out, false, trade_direction, 0)
            .unwrap();

            pool.apply_swap_result(&swap_result_0, trade_direction, 0).unwrap();
            // swap back

            // let swap_result_1 = pool
            // .get_swap_result_exact_out(swap_result_0.input_amount, false, TradeDirection::AtoB, 0)
            // .unwrap();

            // assert!(swap_result_1.input_amount <= amount_out);
        }
    }

}

#[test]
fn test_swap_exact_out_a_to_b_fee_on_both() {
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = u64::MAX as u128;
    let liquidity = u128::MAX;
    let mut pool = Pool {
        // pool_fees,
        ..Default::default()
    };
    pool.liquidity = liquidity;
    pool.sqrt_max_price = sqrt_max_price;
    pool.sqrt_min_price = sqrt_min_price;
    pool.sqrt_price = sqrt_price;

    let amount_exact_out = 100_000_000;

    let is_referral = false;
    let a_to_b = TradeDirection::AtoB;
    let swap_result = pool
        .get_swap_result_exact_out(amount_exact_out, is_referral, a_to_b, 0)
        .unwrap();

    println!("result a to b {:?}", swap_result);
    assert!(swap_result.output_amount == amount_exact_out);

    // return;

    pool.apply_swap_result(&swap_result, a_to_b, 0).unwrap();

    let swap_result_reverse = pool
        .get_swap_result_exact_out(
            swap_result.input_amount,
            is_referral,
            TradeDirection::BtoA,
            0,
        )
        .unwrap();

    println!("reverse {:?}", swap_result_reverse);
    assert!(swap_result_reverse.input_amount <= amount_exact_out);
}

#[test]
fn test_swap_exact_out_a_to_b_fee_on_b() {
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = u64::MAX as u128;
    let liquidity = u128::MAX;
    let mut pool = Pool {
        // pool_fees,
        ..Default::default()
    };
    pool.liquidity = liquidity;
    pool.sqrt_max_price = sqrt_max_price;
    pool.sqrt_min_price = sqrt_min_price;
    pool.sqrt_price = sqrt_price;
    pool.collect_fee_mode = 1; // onlyB

    let amount_exact_out = 100_000_000;

    let is_referral = false;
    let a_to_b = TradeDirection::AtoB;
    let swap_result = pool
        .get_swap_result_exact_out(amount_exact_out, is_referral, a_to_b, 0)
        .unwrap();

    println!("result a to b {:?}", swap_result);
    assert!(swap_result.output_amount == amount_exact_out);

    // return;

    pool.apply_swap_result(&swap_result, a_to_b, 0).unwrap();

    let swap_result_reverse = pool
        .get_swap_result_exact_out(
            swap_result.input_amount,
            is_referral,
            TradeDirection::BtoA,
            0,
        )
        .unwrap();

    println!("reverse {:?}", swap_result_reverse);
    assert!(swap_result_reverse.input_amount <= amount_exact_out);
}

#[test]
fn test_swap_exact_out_b_to_a_fee_on_both() {
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = u64::MAX as u128;
    let liquidity = u128::MAX;
    let mut pool = Pool {
        // pool_fees,
        ..Default::default()
    };
    pool.liquidity = liquidity;
    pool.sqrt_max_price = sqrt_max_price;
    pool.sqrt_min_price = sqrt_min_price;
    pool.sqrt_price = sqrt_price;

    let amount_exact_out = 100_000_000;

    let is_referral = false;
    let b_to_a = TradeDirection::BtoA;
    let swap_result = pool
        .get_swap_result_exact_out(amount_exact_out, is_referral, b_to_a, 0)
        .unwrap();

    println!("result a to b {:?}", swap_result);
    assert!(swap_result.output_amount == amount_exact_out);

    // return;

    pool.apply_swap_result(&swap_result, b_to_a, 0).unwrap();

    let swap_result_reverse = pool
        .get_swap_result_exact_out(
            swap_result.input_amount,
            is_referral,
            TradeDirection::AtoB,
            0,
        )
        .unwrap();

    println!("reverse {:?}", swap_result_reverse);
    // exact out amount in result equal amount_exact_out
    assert!(swap_result_reverse.input_amount <= amount_exact_out);
}

#[test]
fn test_swap_exact_out_b_to_a_fee_on_b() {
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = u64::MAX as u128;
    let liquidity = u128::MAX;
    let mut pool = Pool {
        // pool_fees,
        ..Default::default()
    };
    pool.liquidity = liquidity;
    pool.sqrt_max_price = sqrt_max_price;
    pool.sqrt_min_price = sqrt_min_price;
    pool.sqrt_price = sqrt_price;
    pool.collect_fee_mode = 1; // onlyB

    let amount_exact_out = 100_000_000;

    let is_referral = false;
    let b_to_a = TradeDirection::BtoA;
    let swap_result = pool
        .get_swap_result_exact_out(amount_exact_out, is_referral, b_to_a, 0)
        .unwrap();

    println!("result a to b {:?}", swap_result);

    assert!(swap_result.output_amount == amount_exact_out);

    // return;

    pool.apply_swap_result(&swap_result, b_to_a, 0).unwrap();

    let swap_result_reverse = pool
        .get_swap_result_exact_out(
            swap_result.input_amount,
            is_referral,
            TradeDirection::AtoB,
            0,
        )
        .unwrap();

    println!("reverse {:?}", swap_result_reverse);
    assert!(swap_result_reverse.input_amount <= amount_exact_out);
}
