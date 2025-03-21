use crate::{
    constants::{MAX_SQRT_PRICE, MIN_SQRT_PRICE},
    params::swap::TradeDirection,
    state::{
        fee::{BaseFeeStruct, PoolFeesStruct},
        Pool, Position,
    },
    tests::LIQUIDITY_MAX,
    u128x128_math::Rounding,
};
use proptest::{bool::ANY, prelude::*};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100, .. ProptestConfig::default()
    })]
    #[test]
    fn test_reserve_wont_loss(
        sqrt_price in MIN_SQRT_PRICE..=MAX_SQRT_PRICE,
        liquidity_delta in 1..=LIQUIDITY_MAX / 1000,
        is_referral in ANY,
        amount_in_a in 1..=u32::MAX as u64,
        amount_in_b in 1..=u32::MAX as u64,
    ) {

        let pool_fees = PoolFeesStruct {
            base_fee: BaseFeeStruct{
                cliff_fee_numerator: 1_000_000,
                ..Default::default()
            }, //1%
            protocol_fee_percent: 20,
            partner_fee_percent: 50,
            referral_fee_percent: 20,
            ..Default::default()
        };

        let mut pool = Pool {
            pool_fees,
            sqrt_price,
            sqrt_min_price: MIN_SQRT_PRICE,
            sqrt_max_price: MAX_SQRT_PRICE,
            ..Default::default()
        };

        let mut reserve = PoolReserve::default();

        let mut position = Position::default();

        let mut swap_count = 0;
        for _i in 0..100 {
            //random action
            execute_add_liquidity(&mut reserve, &mut pool, &mut position, liquidity_delta);

            if execute_swap_liquidity(&mut reserve, &mut pool, amount_in_a, is_referral, TradeDirection::AtoB){
                swap_count += 1;
            }

            if execute_swap_liquidity(&mut reserve, &mut pool, amount_in_b, is_referral, TradeDirection::BtoA) {
                swap_count += 1;
            }


            execute_remove_liquidity(&mut reserve, &mut pool, &mut position, liquidity_delta/2);
        }

        let total_liquidity = position.unlocked_liquidity;
        execute_remove_liquidity(&mut reserve, &mut pool, &mut position, total_liquidity);

        assert!(pool.liquidity == 0);
        assert!(position.unlocked_liquidity == 0);
        assert!(position.fee_b_pending <= reserve.amount_b);
        assert!(position.fee_a_pending <= reserve.amount_a);

        println!("{:?}", reserve);
        println!("{:?}", position);
        println!("{:?}", pool);
        println!("swap_count {}", swap_count);
    }
}

#[test]
fn test_reserve_wont_lost_single() {
    let sqrt_price = 4295048016;
    let liquidity_delta = 256772808979395951;
    let is_referral = false;
    let trade_direction = false;
    let amount_in = 1;

    let pool_fees = PoolFeesStruct {
        base_fee: BaseFeeStruct {
            cliff_fee_numerator: 1_000_000,
            ..Default::default()
        }, //1%
        protocol_fee_percent: 20,
        partner_fee_percent: 50,
        referral_fee_percent: 20,
        ..Default::default()
    };
    let mut pool = Pool {
        pool_fees,
        sqrt_price,
        sqrt_min_price: MIN_SQRT_PRICE,
        sqrt_max_price: MAX_SQRT_PRICE,
        ..Default::default()
    };

    let mut reserve = PoolReserve::default();

    let mut position = Position::default();

    let mut swap_count = 0;
    for _i in 0..100 {
        // println!("i {}", i);
        //random action
        execute_add_liquidity(&mut reserve, &mut pool, &mut position, liquidity_delta);

        if trade_direction {
            if execute_swap_liquidity(
                &mut reserve,
                &mut pool,
                amount_in,
                is_referral,
                TradeDirection::AtoB,
            ) {
                swap_count += 1;
            }
        } else {
            if execute_swap_liquidity(
                &mut reserve,
                &mut pool,
                amount_in,
                is_referral,
                TradeDirection::BtoA,
            ) {
                swap_count += 1;
            }
        }

        execute_remove_liquidity(&mut reserve, &mut pool, &mut position, liquidity_delta / 2);
    }

    let total_liquidity = position.unlocked_liquidity;
    println!(
        "swap count {} total liquidity {}",
        swap_count, total_liquidity
    );

    execute_remove_liquidity(&mut reserve, &mut pool, &mut position, total_liquidity);

    assert!(pool.liquidity == 0);
    assert!(position.unlocked_liquidity == 0);

    println!("{:?}", reserve);
    println!("{:?}", position);
    println!("{:?}", pool);
    assert!(position.fee_b_pending <= reserve.amount_b);
    assert!(position.fee_a_pending <= reserve.amount_a);
}

#[derive(Debug, Default)]
pub struct PoolReserve {
    pub amount_a: u64,
    pub amount_b: u64,
}

fn execute_add_liquidity(
    reserve: &mut PoolReserve,
    pool: &mut Pool,
    position: &mut Position,
    liquidity_delta: u128,
) {
    let result = pool
        .get_amounts_for_modify_liquidity(liquidity_delta, Rounding::Up)
        .unwrap();

    pool.apply_add_liquidity(position, liquidity_delta).unwrap();

    reserve.amount_a = reserve.amount_a.checked_add(result.amount_a).unwrap();
    reserve.amount_b = reserve.amount_b.checked_add(result.amount_b).unwrap();
}

fn execute_remove_liquidity(
    reserve: &mut PoolReserve,
    pool: &mut Pool,
    position: &mut Position,
    liquidity_delta: u128,
) {
    let result = pool
        .get_amounts_for_modify_liquidity(liquidity_delta, Rounding::Down)
        .unwrap();

    pool.apply_remove_liquidity(position, liquidity_delta)
        .unwrap();

    reserve.amount_a = reserve.amount_a.checked_sub(result.amount_a).unwrap();
    reserve.amount_b = reserve.amount_b.checked_sub(result.amount_b).unwrap();
}

fn execute_swap_liquidity(
    reserve: &mut PoolReserve,
    pool: &mut Pool,
    amount_in: u64,
    is_referral: bool,
    trade_direction: TradeDirection,
) -> bool {
    let max_amount_in = pool.get_max_amount_in(trade_direction).unwrap();
    if amount_in > max_amount_in {
        return false;
    }
    let swap_result = pool
        .get_swap_result_exact_in(amount_in, is_referral, trade_direction, 0)
        .unwrap();

    pool.apply_swap_result(&swap_result, trade_direction, 0)
        .unwrap();

    match trade_direction {
        TradeDirection::AtoB => {
            reserve.amount_a = reserve.amount_a.checked_add(amount_in).unwrap();
            reserve.amount_b = reserve
                .amount_b
                .checked_sub(swap_result.output_amount)
                .unwrap();
        }
        TradeDirection::BtoA => {
            reserve.amount_b = reserve.amount_b.checked_add(amount_in).unwrap();
            reserve.amount_a = reserve
                .amount_a
                .checked_sub(swap_result.output_amount)
                .unwrap();
        }
    }
    return true;
}
