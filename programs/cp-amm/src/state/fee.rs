use std::u64;

use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use static_assertions::const_assert_eq;

use crate::{
    constants::{
        fee::{FEE_DENOMINATOR, MAX_FEE_NUMERATOR},
        BASIS_POINT_MAX,
    },
    fee_math::get_fee_in_period,
    safe_math::SafeMath,
    u128x128_math::Rounding,
    utils_math::safe_mul_div_cast_u64,
    PoolError,
};

/// Encodes all results of swapping
#[derive(Debug, PartialEq)]
pub struct FeeOnAmountResult {
    pub amount: u64,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

/// collect fee mode
#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    IntoPrimitive,
    TryFromPrimitive,
    AnchorDeserialize,
    AnchorSerialize,
)]

// https://www.desmos.com/calculator/oxdndn2xdx
pub enum FeeSchedulerMode {
    // fee = cliff_fee_numerator - passed_period * reduction_factor
    Linear,
    // fee = cliff_fee_numerator * (1-reduction_factor/10_000)^passed_period
    Exponential,
}

#[zero_copy]
/// Information regarding fee charges
/// trading_fee = amount * trade_fee_numerator / denominator
/// protocol_fee = trading_fee * protocol_fee_percentage / 100
/// referral_fee = protocol_fee * referral_percentage / 100
/// partner_fee = (protocol_fee - referral_fee) * partner_fee_percentage / denominator
#[derive(Debug, InitSpace, Default)]
pub struct PoolFeesStruct {
    /// Trade fees are extra token amounts that are held inside the token
    /// accounts during a trade, making the value of liquidity tokens rise.
    /// Trade fee numerator
    pub base_fee: BaseFeeStruct,

    /// Protocol trading fees are extra token amounts that are held inside the token
    /// accounts during a trade, with the equivalent in pool tokens minted to
    /// the protocol of the program.
    /// Protocol trade fee numerator
    pub protocol_fee_percent: u8,
    /// partner fee
    pub partner_fee_percent: u8,
    /// referral fee
    pub referral_fee_percent: u8,
    /// padding
    pub padding_0: [u8; 5],

    /// dynamic fee
    pub dynamic_fee: DynamicFeeStruct,

    /// padding
    pub padding_1: [u64; 2],
}

const_assert_eq!(PoolFeesStruct::INIT_SPACE, 160);

#[zero_copy]
#[derive(Debug, InitSpace, Default)]
pub struct BaseFeeStruct {
    pub cliff_fee_numerator: u64,
    pub fee_scheduler_mode: u8,
    pub padding_0: [u8; 5],
    pub number_of_period: u16,
    pub period_frequency: u64,
    pub reduction_factor: u64,
    pub padding_1: u64,
}

const_assert_eq!(BaseFeeStruct::INIT_SPACE, 40);

impl BaseFeeStruct {
    pub fn get_max_base_fee_numerator(&self) -> u64 {
        self.cliff_fee_numerator
    }
    pub fn get_min_base_fee_numerator(&self) -> Result<u64> {
        // trick to force current_point < activation_point
        self.get_current_base_fee_numerator(0, 1)
    }
    pub fn get_current_base_fee_numerator(
        &self,
        current_point: u64,
        activation_point: u64,
    ) -> Result<u64> {
        if self.period_frequency == 0 {
            return Ok(self.cliff_fee_numerator);
        }
        // can trade before activation point, so it is alpha-vault, we use min fee
        let period = if current_point < activation_point {
            self.number_of_period.into()
        } else {
            let period = current_point
                .safe_sub(activation_point)?
                .safe_div(self.period_frequency)?;
            period.min(self.number_of_period.into())
        };
        let fee_scheduler_mode = FeeSchedulerMode::try_from(self.fee_scheduler_mode)
            .map_err(|_| PoolError::TypeCastFailed)?;

        match fee_scheduler_mode {
            FeeSchedulerMode::Linear => {
                let fee_numerator = self
                    .cliff_fee_numerator
                    .safe_sub(period.safe_mul(self.reduction_factor.into())?)?;
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

impl PoolFeesStruct {
    // in numerator
    pub fn get_total_trading_fee(&self, current_point: u64, activation_point: u64) -> Result<u128> {
        let base_fee_numerator = self
            .base_fee
            .get_current_base_fee_numerator(current_point, activation_point)?;
        let total_fee_numerator = self
            .dynamic_fee
            .get_variable_fee()?
            .safe_add(base_fee_numerator.into())?;
        Ok(total_fee_numerator)
    }

    pub fn get_fee_on_amount(
        &self,
        amount: u64,
        is_referral: bool,
        current_point: u64,
        activation_point: u64,
        is_exact_out: bool,
    ) -> Result<FeeOnAmountResult> {
        let trade_fee_numerator = self.get_total_trading_fee(current_point, activation_point)?;
        let trade_fee_numerator = if trade_fee_numerator > MAX_FEE_NUMERATOR.into() {
            MAX_FEE_NUMERATOR
        } else {
            trade_fee_numerator.try_into().unwrap()
        };
        let (amount_caculated_fee, lp_fee) = if is_exact_out {
            let amount_included_lp_fee = safe_mul_div_cast_u64(
                amount,
                FEE_DENOMINATOR,
                FEE_DENOMINATOR.safe_sub(trade_fee_numerator)?,
                Rounding::Up,
            )?;

            let lp_fee: u64 = safe_mul_div_cast_u64(
                amount_included_lp_fee,
                trade_fee_numerator,
                FEE_DENOMINATOR,
                Rounding::Down,
            )?;
            (amount_included_lp_fee, lp_fee)
        } else {
            let lp_fee: u64 = safe_mul_div_cast_u64(
                amount,
                trade_fee_numerator,
                FEE_DENOMINATOR,
                Rounding::Down,
            )?;
            // update amount
            let amount = amount.safe_sub(lp_fee)?;
            (amount, lp_fee)
        };

        let protocol_fee = safe_mul_div_cast_u64(
            lp_fee,
            self.protocol_fee_percent.into(),
            100,
            Rounding::Down,
        )?;
        // update lp fee
        let lp_fee = lp_fee.safe_sub(protocol_fee)?;

        let referral_fee = if is_referral {
            safe_mul_div_cast_u64(
                protocol_fee,
                self.referral_fee_percent.into(),
                100,
                Rounding::Down,
            )?
        } else {
            0
        };

        let protocol_fee_after_referral_fee = protocol_fee.safe_sub(referral_fee)?;
        let partner_fee = safe_mul_div_cast_u64(
            protocol_fee_after_referral_fee,
            self.partner_fee_percent.into(),
            100,
            Rounding::Down,
        )?;

        let protocol_fee = protocol_fee_after_referral_fee.safe_sub(partner_fee)?;

        Ok(FeeOnAmountResult {
            amount: amount_caculated_fee,
            lp_fee,
            protocol_fee,
            partner_fee,
            referral_fee,
        })
    }
}

#[zero_copy]
#[derive(Debug, InitSpace, Default)]
pub struct DynamicFeeStruct {
    pub initialized: u8, // 0, ignore for dynamic fee
    pub padding: [u8; 7],
    pub max_volatility_accumulator: u32,
    pub variable_fee_control: u32,
    pub bin_step: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub last_update_timestamp: u64,
    pub bin_step_u128: u128,
    pub sqrt_price_reference: u128, // reference sqrt price
    pub volatility_accumulator: u128,
    pub volatility_reference: u128, // decayed volatility accumulator
}

const_assert_eq!(DynamicFeeStruct::INIT_SPACE, 96);

impl DynamicFeeStruct {
    // we approximate (1+bin_step)^bin_id = 1 + bin_step * bin_id
    pub fn get_delta_bin_id(
        bin_step_u128: u128,
        sqrt_price_a: u128,
        sqrt_price_b: u128,
    ) -> Result<u128> {
        let delta_id = if sqrt_price_a > sqrt_price_b {
            sqrt_price_a
                .safe_sub(sqrt_price_b)?
                .safe_div(bin_step_u128)?
        } else {
            sqrt_price_b
                .safe_sub(sqrt_price_a)?
                .safe_div(bin_step_u128)?
        };
        Ok(delta_id.safe_mul(2)?) // mul 2 because we are using sqrt price
    }
    pub fn update_volatility_accumulator(&mut self, sqrt_price: u128) -> Result<()> {
        let delta_price =
            Self::get_delta_bin_id(self.bin_step_u128, sqrt_price, self.sqrt_price_reference)?;

        let volatility_accumulator = self
            .volatility_reference
            .safe_add(delta_price.safe_mul(BASIS_POINT_MAX.into())?)?;

        self.volatility_accumulator = std::cmp::min(
            volatility_accumulator,
            self.max_volatility_accumulator.into(),
        );
        Ok(())
    }

    pub fn update_references(
        &mut self,
        sqrt_price_current: u128,
        current_timestamp: u64,
    ) -> Result<()> {
        let elapsed = current_timestamp.safe_sub(self.last_update_timestamp)?;
        // Not high frequency trade
        if elapsed >= self.filter_period as u64 {
            // Update sqrt of last transaction
            self.sqrt_price_reference = sqrt_price_current;
            // filter period < t < decay_period. Decay time window.
            if elapsed < self.decay_period as u64 {
                let volatility_reference = self
                    .volatility_accumulator
                    .safe_mul(self.reduction_factor.into())?
                    .safe_div(BASIS_POINT_MAX.into())?;

                self.volatility_reference = volatility_reference;
            }
            // Out of decay time window
            else {
                self.volatility_reference = 0;
            }
        }
        Ok(())
    }

    pub fn is_dynamic_fee_enable(&self) -> bool {
        self.initialized != 0
    }

    pub fn get_variable_fee(&self) -> Result<u128> {
        if self.is_dynamic_fee_enable() {
            let square_vfa_bin: u128 = self
                .volatility_accumulator
                .safe_mul(self.bin_step.into())?
                .checked_pow(2)
                .unwrap();
            // Variable fee control, volatility accumulator, bin step are in basis point unit (10_000)
            // This is 1e20. Which > 1e9. Scale down it to 1e9 unit and ceiling the remaining.
            let v_fee = square_vfa_bin.safe_mul(self.variable_fee_control.into())?;

            let scaled_v_fee = v_fee.safe_add(99_999_999_999)?.safe_div(100_000_000_000)?;

            Ok(scaled_v_fee)
        } else {
            Ok(0)
        }
    }
}
