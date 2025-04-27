use anyhow::{ensure, Context, Ok, Result};
use cp_amm::{
    params::swap::TradeDirection,
    state::{fee::FeeMode, Pool, SwapResult},
    ActivationType,
};

pub fn get_quote(
    pool: &Pool,
    current_timestamp: u64,
    current_slot: u64,
    actual_amount_in: u64,
    a_to_b: bool,
    has_referral: bool,
) -> Result<SwapResult> {
    ensure!(actual_amount_in > 0, "amount is zero");

    let result = if pool.pool_fees.dynamic_fee.is_dynamic_fee_enable() {
        let mut pool = *pool;
        pool.update_pre_swap(current_timestamp)?;
        get_internal_quote(
            &pool,
            current_timestamp,
            current_slot,
            actual_amount_in,
            a_to_b,
            has_referral,
        )
    } else {
        get_internal_quote(
            pool,
            current_timestamp,
            current_slot,
            actual_amount_in,
            a_to_b,
            has_referral,
        )
    };

    result
}

fn get_internal_quote(
    pool: &Pool,
    current_timestamp: u64,
    current_slot: u64,
    actual_amount_in: u64,
    a_to_b: bool,
    has_referral: bool,
) -> Result<SwapResult> {
    let activation_type =
        ActivationType::try_from(pool.activation_type).context("invalid activation type")?;

    let current_point = match activation_type {
        ActivationType::Slot => current_slot,
        ActivationType::Timestamp => current_timestamp,
    };

    let trade_direction = if a_to_b {
        TradeDirection::AtoB
    } else {
        TradeDirection::BtoA
    };

    let fee_mode =
        &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, has_referral, false)?;

    let swap_result = pool.get_swap_result(
        actual_amount_in,
        fee_mode,
        trade_direction,
        current_point,
        false,
    )?;

    Ok(swap_result)
}
