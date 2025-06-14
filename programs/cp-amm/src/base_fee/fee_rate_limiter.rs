use crate::{
    activation_handler::ActivationType,
    constants::{
        fee::{FEE_DENOMINATOR, MAX_FEE_NUMERATOR_V1, MIN_FEE_NUMERATOR},
        MAX_RATE_LIMITER_DURATION_IN_SECONDS, MAX_RATE_LIMITER_DURATION_IN_SLOTS,
    },
    params::{fee_parameters::to_numerator, swap::TradeDirection},
    safe_math::SafeMath,
    state::CollectFeeMode,
    u128x128_math::Rounding,
    utils_math::safe_mul_div_cast_u64,
    PoolError,
};

use super::BaseFeeHandler;
use anchor_lang::prelude::*;
use num::Integer;
use ruint::aliases::U256;

/// we denote reference_amount = x0, cliff_fee_numerator = c, fee_increment = i
/// if input_amount <= x0, then fee = input_amount * c
///
/// if input_amount > x0, then input_amount = x0 + (a * x0 + b)
/// if a < max_index
/// then fee = x0 * c + x0 * (c + i) + .... + x0 * (c + i*a) + b * (c + i * (a+1))
/// then fee = x0 * (c + c*a + i*a*(a+1)/2) + b * (c + i * (a+1))
///
/// if a >= max_index
/// if a = max_index + d, input_amount = x0 + max_index * x0 + (d * x0 + b)
/// then fee = x0 * (c + c*max_index + i*max_index*(max_index+1)/2) + (d * x0 + b) * MAX_FEE
#[derive(Debug, Default)]
pub struct FeeRateLimiter {
    pub cliff_fee_numerator: u64,
    pub fee_increment_bps: u16,
    pub max_limiter_duration: u64,
    pub reference_amount: u64,
}

impl FeeRateLimiter {
    pub fn is_rate_limiter_applied(
        &self,
        current_point: u64,
        activation_point: u64,
        trade_direction: TradeDirection,
    ) -> Result<bool> {
        if self.is_zero_rate_limiter() {
            return Ok(false);
        }

        // only handle for the case B to A and collect fee mode in token B
        if trade_direction == TradeDirection::AtoB {
            return Ok(false);
        }

        // it means whitelisted vault is buying
        if current_point < activation_point {
            return Ok(false);
        }

        let last_effective_rate_limiter_point =
            u128::from(activation_point).safe_add(self.max_limiter_duration.into())?;
        if u128::from(current_point) > last_effective_rate_limiter_point {
            return Ok(false);
        }
        Ok(true)
    }

    fn is_zero_rate_limiter(&self) -> bool {
        self.reference_amount == 0 && self.max_limiter_duration == 0 && self.fee_increment_bps == 0
    }

    fn is_non_zero_rate_limiter(&self) -> bool {
        self.reference_amount != 0 && self.max_limiter_duration != 0 && self.fee_increment_bps != 0
    }

    pub fn get_max_index(&self) -> Result<u64> {
        let delta_numerator = MAX_FEE_NUMERATOR_V1.safe_sub(self.cliff_fee_numerator)?;
        let fee_increment_numerator =
            to_numerator(self.fee_increment_bps.into(), FEE_DENOMINATOR.into())?;
        let max_index = delta_numerator.safe_div(fee_increment_numerator)?;
        Ok(max_index)
    }

    // export function for testing
    pub fn get_fee_numerator_from_amount(&self, input_amount: u64) -> Result<u64> {
        let fee_numerator = if input_amount <= self.reference_amount {
            self.cliff_fee_numerator
        } else {
            let c = U256::from(self.cliff_fee_numerator);
            let (a, b) = input_amount
                .safe_sub(self.reference_amount)?
                .div_rem(&self.reference_amount);
            let a = U256::from(a);
            let b = U256::from(b);
            let max_index = U256::from(self.get_max_index()?);
            let i = U256::from(to_numerator(
                self.fee_increment_bps.into(),
                FEE_DENOMINATOR.into(),
            )?);
            let x0 = U256::from(self.reference_amount);
            let one = U256::ONE;
            let two = U256::from(2);
            // because we all calculate in U256, so it is safe to avoid safe math
            let trading_fee_numerator = if a < max_index {
                let numerator_1 = c + c * a + i * a * (a + one) / two;
                let numerator_2 = c + i * (a + one);
                let first_fee = x0 * numerator_1;
                let second_fee = b * numerator_2;
                first_fee + second_fee
            } else {
                let numerator_1 = c + c * max_index + i * max_index * (max_index + one) / two;
                let numerator_2 = U256::from(MAX_FEE_NUMERATOR_V1);
                let first_fee = x0 * numerator_1;

                let d = a - max_index;
                let left_amount = d * x0 + b;
                let second_fee = left_amount * numerator_2;
                first_fee + second_fee
            };

            let denominator = U256::from(FEE_DENOMINATOR);
            let trading_fee = (trading_fee_numerator + denominator - one) / denominator;
            let trading_fee = trading_fee
                .try_into()
                .map_err(|_| PoolError::TypeCastFailed)?;

            // reverse to fee numerator
            // input_amount * numerator / FEE_DENOMINATOR = trading_fee
            // then numerator = trading_fee * FEE_DENOMINATOR / input_amount
            let fee_numerator =
                safe_mul_div_cast_u64(trading_fee, FEE_DENOMINATOR, input_amount, Rounding::Up)?;
            fee_numerator
        };

        Ok(fee_numerator)
    }
}

impl BaseFeeHandler for FeeRateLimiter {
    fn validate(
        &self,
        collect_fee_mode: CollectFeeMode,
        activation_type: ActivationType,
    ) -> Result<()> {
        // can only be applied in OnlyB collect fee mode
        require!(
            collect_fee_mode == CollectFeeMode::OnlyB,
            PoolError::InvalidFeeRateLimiter
        );

        if self.is_zero_rate_limiter() {
            return Ok(());
        }

        require!(
            self.is_non_zero_rate_limiter(),
            PoolError::InvalidFeeRateLimiter
        );

        let max_limiter_duration = match activation_type {
            ActivationType::Slot => MAX_RATE_LIMITER_DURATION_IN_SLOTS,
            ActivationType::Timestamp => MAX_RATE_LIMITER_DURATION_IN_SECONDS,
        };
        require!(
            self.max_limiter_duration <= max_limiter_duration,
            PoolError::InvalidFeeRateLimiter
        );

        let fee_increment_numerator =
            to_numerator(self.fee_increment_bps.into(), FEE_DENOMINATOR.into())?;
        require!(
            fee_increment_numerator < FEE_DENOMINATOR,
            PoolError::InvalidFeeRateLimiter
        );

        // that condition is redundant, but is is safe to add this
        require!(
            self.cliff_fee_numerator >= MIN_FEE_NUMERATOR
                && self.cliff_fee_numerator <= MAX_FEE_NUMERATOR_V1,
            PoolError::InvalidFeeRateLimiter
        );

        // validate max fee (more amount, then more fee)
        let min_fee_numerator = self.get_fee_numerator_from_amount(0)?;
        let max_fee_numerator = self.get_fee_numerator_from_amount(u64::MAX)?;
        require!(
            min_fee_numerator >= MIN_FEE_NUMERATOR && max_fee_numerator <= MAX_FEE_NUMERATOR_V1,
            PoolError::InvalidFeeRateLimiter
        );
        Ok(())
    }
    fn get_base_fee_numerator(
        &self,
        current_point: u64,
        activation_point: u64,
        trade_direction: TradeDirection,
        input_amount: u64,
    ) -> Result<u64> {
        if self.is_rate_limiter_applied(current_point, activation_point, trade_direction)? {
            self.get_fee_numerator_from_amount(input_amount)
        } else {
            Ok(self.cliff_fee_numerator)
        }
    }
}
