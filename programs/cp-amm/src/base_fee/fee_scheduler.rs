use crate::{
    activation_handler::ActivationType,
    constants::fee::{FEE_DENOMINATOR, MAX_FEE_NUMERATOR_V1, MIN_FEE_NUMERATOR},
    fee_math::get_fee_in_period,
    math::safe_math::SafeMath,
    params::{fee_parameters::validate_fee_fraction, swap::TradeDirection},
    state::CollectFeeMode,
    PoolError,
};
use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::BaseFeeHandler;

// https://www.desmos.com/calculator/oxdndn2xdx
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum FeeSchedulerMode {
    // fee = cliff_fee_numerator - passed_period * reduction_factor
    Linear,
    // fee = cliff_fee_numerator * (1-reduction_factor/10_000)^passed_period
    Exponential,
}

#[derive(Debug, Default)]
pub struct FeeScheduler {
    pub cliff_fee_numerator: u64,
    pub number_of_period: u16,
    pub period_frequency: u64,
    pub reduction_factor: u64,
    pub fee_scheduler_mode: u8,
}

impl FeeScheduler {
    pub fn get_max_base_fee_numerator(&self) -> u64 {
        self.cliff_fee_numerator
    }

    pub fn get_min_base_fee_numerator(&self) -> Result<u64> {
        self.get_base_fee_numerator_by_period(self.number_of_period.into())
    }

    fn get_base_fee_numerator_by_period(&self, period: u64) -> Result<u64> {
        let period = period.min(self.number_of_period.into());

        let base_fee_mode = FeeSchedulerMode::try_from(self.fee_scheduler_mode)
            .map_err(|_| PoolError::TypeCastFailed)?;

        match base_fee_mode {
            FeeSchedulerMode::Linear => {
                let fee_numerator = self
                    .cliff_fee_numerator
                    .safe_sub(self.reduction_factor.safe_mul(period)?)?;
                Ok(fee_numerator)
            }
            FeeSchedulerMode::Exponential => {
                let period = u16::try_from(period).map_err(|_| PoolError::MathOverflow)?;
                let fee_numerator =
                    get_fee_in_period(self.cliff_fee_numerator, self.reduction_factor, period)?;
                Ok(fee_numerator)
            }
        }
    }
}

impl BaseFeeHandler for FeeScheduler {
    fn validate(
        &self,
        _collect_fee_mode: CollectFeeMode,
        _activation_type: ActivationType,
    ) -> Result<()> {
        if self.period_frequency != 0 || self.number_of_period != 0 || self.reduction_factor != 0 {
            require!(
                self.number_of_period != 0
                    && self.period_frequency != 0
                    && self.reduction_factor != 0,
                PoolError::InvalidFeeScheduler
            );
        }
        let min_fee_numerator = self.get_min_base_fee_numerator()?;
        let max_fee_numerator = self.get_max_base_fee_numerator();
        validate_fee_fraction(min_fee_numerator, FEE_DENOMINATOR)?;
        validate_fee_fraction(max_fee_numerator, FEE_DENOMINATOR)?;
        require!(
            min_fee_numerator >= MIN_FEE_NUMERATOR && max_fee_numerator <= MAX_FEE_NUMERATOR_V1,
            PoolError::ExceedMaxFeeBps
        );
        Ok(())
    }
    fn get_base_fee_numerator(
        &self,
        current_point: u64,
        activation_point: u64,
        _trade_direction: TradeDirection,
        _input_amount: u64,
    ) -> Result<u64> {
        if self.period_frequency == 0 {
            return Ok(self.cliff_fee_numerator);
        }
        // it means alpha-vault is buying
        let period = if current_point < activation_point {
            self.number_of_period.into()
        } else {
            let period = current_point
                .safe_sub(activation_point)?
                .safe_div(self.period_frequency)?;
            period.min(self.number_of_period.into())
        };
        self.get_base_fee_numerator_by_period(period)
    }
}
