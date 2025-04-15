//! Fees module includes information about fee charges
use crate::constants::fee::{
    CUSTOMIZABLE_HOST_FEE_PERCENT, CUSTOMIZABLE_PROTOCOL_FEE_PERCENT, FEE_DENOMINATOR,
    MAX_BASIS_POINT, MAX_FEE_BPS_DEFAULT, MIN_FEE_BPS, MIN_FEE_NUMERATOR,
};
use crate::constants::{BASIS_POINT_MAX, BIN_STEP_BPS_DEFAULT, BIN_STEP_BPS_U128_DEFAULT, U24_MAX};
use crate::error::PoolError;
use crate::fee_math::get_fee_in_period;
use crate::safe_math::SafeMath;
use crate::state::fee::{BaseFeeStruct, DynamicFeeStruct, FeeSchedulerMode, PoolFeesStruct};
use crate::state::{BaseFeeConfig, DynamicFeeConfig, PoolFeesConfig};
use anchor_lang::prelude::*;

use super::swap::TradeDirection;

/// Information regarding fee charges
#[derive(Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize, InitSpace, Default)]
pub struct PoolFeeParameters {
    /// Base fee
    pub base_fee: BaseFeeParameters,
    /// Protocol trade fee percent
    pub protocol_fee_percent: u8,
    /// partner fee percent
    pub partner_fee_percent: u8,
    /// referral fee percent
    pub referral_fee_percent: u8,
    /// max_fee_bps
    pub max_fee_bps: u64,
    /// dynamic fee
    pub dynamic_fee: Option<DynamicFeeParameters>,
}

#[derive(Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize, InitSpace, Default)]
pub struct BaseFeeParameters {
    pub cliff_fee_numerator: u64,
    pub number_of_period: u16,
    pub period_frequency: u64,
    pub reduction_factor: u64,
    pub fee_scheduler_mode: u8,
}

impl BaseFeeParameters {
    pub fn get_max_base_fee_numerator(&self) -> u64 {
        self.cliff_fee_numerator
    }
    pub fn get_min_base_fee_numerator(&self) -> Result<u64> {
        let fee_scheduler_mode = FeeSchedulerMode::try_from(self.fee_scheduler_mode)
            .map_err(|_| PoolError::TypeCastFailed)?;
        match fee_scheduler_mode {
            FeeSchedulerMode::Linear => {
                let fee_numerator = self.cliff_fee_numerator.safe_sub(
                    self.reduction_factor
                        .safe_mul(self.number_of_period.into())?,
                )?;
                Ok(fee_numerator)
            }
            FeeSchedulerMode::Exponential => {
                let fee_numerator = get_fee_in_period(
                    self.cliff_fee_numerator,
                    self.reduction_factor,
                    self.number_of_period,
                )?;
                Ok(fee_numerator)
            }
        }
    }

    fn validate(&self, max_fee_numerator: u64) -> Result<()> {
        let min_base_fee_numerator = self.get_min_base_fee_numerator()?;
        let max_base_fee_numerator = self.get_max_base_fee_numerator();
        validate_fee_fraction(min_base_fee_numerator, FEE_DENOMINATOR)?;
        validate_fee_fraction(max_base_fee_numerator, FEE_DENOMINATOR)?;
        require!(
            min_base_fee_numerator >= MIN_FEE_NUMERATOR
                && max_base_fee_numerator <= max_fee_numerator,
            PoolError::ExceedMaxFeeBps
        );
        Ok(())
    }
    fn to_base_fee_struct(&self) -> BaseFeeStruct {
        BaseFeeStruct {
            cliff_fee_numerator: self.cliff_fee_numerator,
            number_of_period: self.number_of_period,
            period_frequency: self.period_frequency,
            reduction_factor: self.reduction_factor,
            fee_scheduler_mode: self.fee_scheduler_mode,
            ..Default::default()
        }
    }

    pub fn to_base_fee_config(&self) -> BaseFeeConfig {
        BaseFeeConfig {
            cliff_fee_numerator: self.cliff_fee_numerator,
            number_of_period: self.number_of_period,
            period_frequency: self.period_frequency,
            reduction_factor: self.reduction_factor,
            fee_scheduler_mode: self.fee_scheduler_mode,
            ..Default::default()
        }
    }
}

impl PoolFeeParameters {
    pub fn to_pool_fees_config(&self) -> PoolFeesConfig {
        let &PoolFeeParameters {
            base_fee,
            protocol_fee_percent,
            partner_fee_percent,
            referral_fee_percent,
            max_fee_bps,
            dynamic_fee,
        } = self;
        if let Some(dynamic_fee) = dynamic_fee {
            PoolFeesConfig {
                base_fee: base_fee.to_base_fee_config(),
                protocol_fee_percent,
                partner_fee_percent,
                referral_fee_percent,
                max_fee_bps,
                max_fee_numerator: to_max_fee_numerator(max_fee_bps).unwrap(),
                dynamic_fee: dynamic_fee.to_dynamic_fee_config(),
                ..Default::default()
            }
        } else {
            PoolFeesConfig {
                base_fee: base_fee.to_base_fee_config(),
                protocol_fee_percent,
                partner_fee_percent,
                referral_fee_percent,
                max_fee_bps,
                max_fee_numerator: to_max_fee_numerator(max_fee_bps).unwrap(),
                ..Default::default()
            }
        }
    }
    pub fn to_pool_fees_struct(&self) -> PoolFeesStruct {
        let &PoolFeeParameters {
            base_fee,
            protocol_fee_percent,
            partner_fee_percent,
            referral_fee_percent,
            max_fee_bps,
            dynamic_fee,
        } = self;
        if let Some(dynamic_fee) = dynamic_fee {
            PoolFeesStruct {
                base_fee: base_fee.to_base_fee_struct(),
                protocol_fee_percent,
                partner_fee_percent,
                referral_fee_percent,
                max_fee_bps,
                max_fee_numerator: to_max_fee_numerator(max_fee_bps).unwrap(),
                dynamic_fee: dynamic_fee.to_dynamic_fee_struct(),
                ..Default::default()
            }
        } else {
            PoolFeesStruct {
                base_fee: base_fee.to_base_fee_struct(),
                protocol_fee_percent,
                partner_fee_percent,
                referral_fee_percent,
                max_fee_bps,
                max_fee_numerator: to_max_fee_numerator(max_fee_bps).unwrap(),
                ..Default::default()
            }
        }
    }
}

#[derive(Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize, InitSpace, Default)]
pub struct DynamicFeeParameters {
    pub bin_step: u16,
    pub bin_step_u128: u128,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub max_volatility_accumulator: u32,
    pub variable_fee_control: u32,
}

impl DynamicFeeParameters {
    fn to_dynamic_fee_config(&self) -> DynamicFeeConfig {
        DynamicFeeConfig {
            initialized: 1,
            bin_step: self.bin_step,
            filter_period: self.filter_period,
            decay_period: self.decay_period,
            reduction_factor: self.reduction_factor,
            bin_step_u128: self.bin_step_u128,
            max_volatility_accumulator: self.max_volatility_accumulator,
            variable_fee_control: self.variable_fee_control,
            ..Default::default()
        }
    }
    fn to_dynamic_fee_struct(&self) -> DynamicFeeStruct {
        DynamicFeeStruct {
            initialized: 1,
            bin_step: self.bin_step,
            bin_step_u128: self.bin_step_u128,
            filter_period: self.filter_period,
            decay_period: self.decay_period,
            reduction_factor: self.reduction_factor,
            max_volatility_accumulator: self.max_volatility_accumulator,
            variable_fee_control: self.variable_fee_control,
            ..Default::default()
        }
    }
    pub fn validate(&self) -> Result<()> {
        // force all bin_step as 1 bps for first version
        require!(
            self.bin_step == BIN_STEP_BPS_DEFAULT,
            PoolError::InvalidInput
        );
        require!(
            self.bin_step_u128 == BIN_STEP_BPS_U128_DEFAULT,
            PoolError::InvalidInput
        );

        // filter period < t < decay period
        require!(
            self.filter_period < self.decay_period,
            PoolError::InvalidInput
        );

        // reduction factor decide the decay rate of variable fee, max reduction_factor is BASIS_POINT_MAX = 100% reduction
        require!(
            self.reduction_factor <= BASIS_POINT_MAX as u16,
            PoolError::InvalidInput
        );

        // prevent program overflow
        require!(
            self.variable_fee_control <= U24_MAX,
            PoolError::InvalidInput
        );
        require!(
            self.max_volatility_accumulator <= U24_MAX,
            PoolError::InvalidInput
        );

        Ok(())
    }
}

/// Helper function for calculating swap fee
pub fn calculate_fee(
    token_amount: u128,
    fee_numerator: u128,
    fee_denominator: u128,
) -> Option<u128> {
    if fee_numerator == 0 || token_amount == 0 {
        Some(0)
    } else {
        let fee = token_amount
            .checked_mul(fee_numerator)?
            .checked_div(fee_denominator)?;
        if fee == 0 {
            Some(1) // minimum fee of one token
        } else {
            Some(fee)
        }
    }
}

pub fn validate_fee_fraction(numerator: u64, denominator: u64) -> Result<()> {
    if denominator == 0 || numerator >= denominator {
        Err(PoolError::InvalidFee.into())
    } else {
        Ok(())
    }
}

/// Convert fees numerator and denominator to BPS. Minimum 1 bps, Maximum 10_000 bps. 0.01% -> 100%
pub fn to_bps(numerator: u128, denominator: u128) -> Option<u64> {
    let bps = numerator
        .checked_mul(MAX_BASIS_POINT.into())?
        .checked_div(denominator)?;
    bps.try_into().ok()
}

pub fn to_max_fee_numerator(max_fee_bps: u64) -> Result<u64> {
    let max_fee_numerator = max_fee_bps
        .safe_mul(FEE_DENOMINATOR)?
        .safe_div(MAX_BASIS_POINT)?;

    Ok(max_fee_numerator)
}

impl PoolFeeParameters {
    /// Validate that the fees are reasonable
    pub fn validate(&self) -> Result<()> {
        require!(
            self.max_fee_bps > MIN_FEE_BPS && self.max_fee_bps <= MAX_FEE_BPS_DEFAULT,
            PoolError::InvalidFee
        );

        let max_fee_numerator = to_max_fee_numerator(self.max_fee_bps).unwrap();
        self.base_fee.validate(max_fee_numerator)?;

        validate_fee_fraction(self.protocol_fee_percent.into(), 100)?;
        validate_fee_fraction(self.partner_fee_percent.into(), 100)?;
        validate_fee_fraction(self.referral_fee_percent.into(), 100)?;

        if let Some(dynamic_fee) = self.dynamic_fee {
            dynamic_fee.validate()?;
        }
        Ok(())
    }

    pub fn validate_for_customizable_pool(&self) -> Result<()> {
        require!(
            self.protocol_fee_percent == CUSTOMIZABLE_PROTOCOL_FEE_PERCENT,
            PoolError::InvalidParameters
        );
        require!(
            self.referral_fee_percent == CUSTOMIZABLE_HOST_FEE_PERCENT,
            PoolError::InvalidParameters
        );
        require!(self.partner_fee_percent == 0, PoolError::InvalidParameters);
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize, InitSpace, Default)]
pub struct PartnerInfo {
    pub fee_percent: u8,
    pub partner_authority: Pubkey,
    pub pending_fee_a: u64,
    pub pending_fee_b: u64,
}

impl PartnerInfo {
    pub fn have_partner(&self) -> bool {
        self.partner_authority != Pubkey::default()
    }

    pub fn validate(&self) -> Result<()> {
        if !self.have_partner() {
            require!(self.fee_percent == 0, PoolError::InvalidFee);
        }

        Ok(())
    }

    pub fn accrue_partner_fees(
        &mut self,
        protocol_fee: u64,
        trade_direction: TradeDirection,
    ) -> Result<()> {
        if self.fee_percent > 0 {
            let partner_profit = protocol_fee
                .safe_mul(self.fee_percent.into())?
                .safe_div(100)?;

            match trade_direction {
                TradeDirection::AtoB => {
                    self.pending_fee_a = self.pending_fee_a.safe_add(partner_profit)?;
                }
                TradeDirection::BtoA => {
                    self.pending_fee_b = self.pending_fee_b.safe_add(partner_profit)?;
                }
            }
        }
        Ok(())
    }

    pub fn claim_fees(&mut self, max_amount_a: u64, max_amount_b: u64) -> Result<(u64, u64)> {
        let claimable_amount_a = max_amount_a.min(self.pending_fee_a);
        let claimable_amount_b = max_amount_b.min(self.pending_fee_b);

        self.pending_fee_a = self.pending_fee_a.safe_sub(claimable_amount_a)?;
        self.pending_fee_b = self.pending_fee_b.safe_sub(claimable_amount_b)?;

        Ok((claimable_amount_a, claimable_amount_b))
    }
}
