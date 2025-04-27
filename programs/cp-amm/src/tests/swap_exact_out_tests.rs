use std::{u128, u64};

use crate::{
    constants::{MAX_SQRT_PRICE, MIN_SQRT_PRICE},
    params::swap::TradeDirection,
    state::{fee::FeeMode, Pool},
    tests::LIQUIDITY_MAX,
};

#[test]
fn test_reserve_wont_lost_when_swap_exact_out_from_a_to_b() {
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = u64::MAX as u128;
    let liquidity = LIQUIDITY_MAX;
    let mut pool = Pool {
        // pool_fees,
        ..Default::default()
    };

    pool.liquidity = liquidity;
    pool.sqrt_max_price = sqrt_max_price;
    pool.sqrt_min_price = sqrt_min_price;
    pool.sqrt_price = sqrt_price;

    let exact_amount_b_out = 100_000_000;
    let trade_direction = TradeDirection::AtoB;
    let fee_mode =
        &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, true, true).unwrap();

    let swap_result = pool
        .get_swap_result(exact_amount_b_out, fee_mode, trade_direction, 0, true)
        .unwrap();

    println!("result {:?}", swap_result);

    pool.apply_swap_result(&swap_result, fee_mode, 0).unwrap();

    let amount_a = swap_result.input_amount;

    let swap_result_reverse = pool
        .get_swap_result(amount_a, fee_mode, TradeDirection::AtoB, 0, false)
        .unwrap();

    println!("reverse {:?}", swap_result_reverse);
    assert!(swap_result_reverse.output_amount <= exact_amount_b_out);
}

#[test]
fn test_reserve_wont_lost_when_swap_exact_out_from_b_to_a() {
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = u64::MAX as u128;
    let liquidity = LIQUIDITY_MAX;
    let mut pool = Pool {
        // pool_fees,
        ..Default::default()
    };

    pool.liquidity = liquidity;
    pool.sqrt_max_price = sqrt_max_price;
    pool.sqrt_min_price = sqrt_min_price;
    pool.sqrt_price = sqrt_price;

    let exact_amount_a_out = 100_000_000;
    let trade_direction = TradeDirection::BtoA;
    let fee_mode =
        &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, true, true).unwrap();

    let swap_result = pool
        .get_swap_result(exact_amount_a_out, fee_mode, trade_direction, 0, true)
        .unwrap();

    println!("result {:?}", swap_result);

    pool.apply_swap_result(&swap_result, fee_mode, 0).unwrap();

    let amount_b: u64 = swap_result.input_amount;

    let swap_result_reverse = pool
        .get_swap_result(amount_b, fee_mode, TradeDirection::BtoA, 0, false)
        .unwrap();

    println!("reverse {:?}", swap_result_reverse);
    assert!(swap_result_reverse.output_amount <= exact_amount_a_out);
}

#[test]
fn test_reverse_swap_exact_out_in_a_to_b() {
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = u64::MAX as u128;
    let liquidity = LIQUIDITY_MAX;
    let mut pool = Pool {
        // pool_fees,
        ..Default::default()
    };

    pool.liquidity = liquidity;
    pool.sqrt_max_price = sqrt_max_price;
    pool.sqrt_min_price = sqrt_min_price;
    pool.sqrt_price = sqrt_price;

    let amount_out = 100_000_000;
    let trade_direction = TradeDirection::AtoB;
    let fee_mode =
        &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, false, true).unwrap();

    let swap_result = pool
        .get_swap_result(amount_out, fee_mode, trade_direction, 0, true)
        .unwrap();

    println!("result {:?}", swap_result);

    let swap_result_reverse = pool
        .get_swap_result(
            swap_result.input_amount,
            fee_mode,
            TradeDirection::AtoB,
            0,
            false,
        )
        .unwrap();

    println!("reverse {:?}", swap_result_reverse);
    assert!(swap_result_reverse.output_amount <= amount_out);
}

#[test]
fn test_reverse_swap_exact_out_in_b_to_a() {
    let sqrt_min_price = MIN_SQRT_PRICE;
    let sqrt_max_price = MAX_SQRT_PRICE;
    let sqrt_price = u64::MAX as u128;
    let liquidity = LIQUIDITY_MAX;
    let mut pool = Pool {
        // pool_fees,
        ..Default::default()
    };

    pool.liquidity = liquidity;
    pool.sqrt_max_price = sqrt_max_price;
    pool.sqrt_min_price = sqrt_min_price;
    pool.sqrt_price = sqrt_price;

    let exact_amout_a_out = 100_000_000;
    let trade_direction = TradeDirection::BtoA;
    let fee_mode =
        &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, false, true).unwrap();

    let swap_result = pool
        .get_swap_result(exact_amout_a_out, fee_mode, trade_direction, 0, true)
        .unwrap();

    println!("result {:?}", swap_result);

    let swap_result_reverse = pool
        .get_swap_result(
            swap_result.input_amount,
            fee_mode,
            TradeDirection::BtoA,
            0,
            false,
        )
        .unwrap();

    println!("reverse {:?}", swap_result_reverse);
    assert!(swap_result_reverse.output_amount <= exact_amout_a_out);
}
