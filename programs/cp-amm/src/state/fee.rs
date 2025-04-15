use std::u64;

use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use static_assertions::const_assert_eq;

use crate::{
    constants::{fee::FEE_DENOMINATOR, BASIS_POINT_MAX, ONE_Q64},
    fee_math::get_fee_in_period,
    params::swap::TradeDirection,
    safe_math::SafeMath,
    u128x128_math::Rounding,
    utils_math::{safe_mul_div_cast_u64, safe_shl_div_cast},
    PoolError,
};

use super::CollectFeeMode;

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

    /// max_fee_bps
    pub max_fee_bps: u64,
    /// max_fee_numerator
    pub max_fee_numerator: u64,

    /// padding
    pub padding_1: [u64; 2],
}

const_assert_eq!(PoolFeesStruct::INIT_SPACE, 176);

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
        has_referral: bool,
        current_point: u64,
        activation_point: u64,
    ) -> Result<FeeOnAmountResult> {
        let trade_fee_numerator = self.get_total_trading_fee(current_point, activation_point)?;
        let trade_fee_numerator = if trade_fee_numerator > self.max_fee_numerator.into() {
            self.max_fee_numerator
        } else {
            trade_fee_numerator.try_into().unwrap()
        };
        let lp_fee: u64 =
            safe_mul_div_cast_u64(amount, trade_fee_numerator, FEE_DENOMINATOR, Rounding::Up)?;
        // update amount
        let amount = amount.safe_sub(lp_fee)?;

        let protocol_fee = safe_mul_div_cast_u64(
            lp_fee,
            self.protocol_fee_percent.into(),
            100,
            Rounding::Down,
        )?;
        // update lp fee
        let lp_fee = lp_fee.safe_sub(protocol_fee)?;

        let referral_fee = if has_referral {
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
            amount,
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
    // we approximate Px / Py = (1 + b) ^ delta_bin  = 1 + b * delta_bin (if b is too small)
    // Ex: (1+1/10000)^ 5000 / (1+5000 * 1/10000) = 1.1 (10% diff if sqrt_price diff is (1+1/10000)^ 5000 = 1.64 times)
    pub fn get_delta_bin_id(
        bin_step_u128: u128,
        sqrt_price_a: u128,
        sqrt_price_b: u128,
    ) -> Result<u128> {
        let (upper_sqrt_price, lower_sqrt_price) = if sqrt_price_a > sqrt_price_b {
            (sqrt_price_a, sqrt_price_b)
        } else {
            (sqrt_price_b, sqrt_price_a)
        };

        let price_ratio: u128 =
            safe_shl_div_cast(upper_sqrt_price, lower_sqrt_price, 64, Rounding::Down)?;

        let delta_bin_id = price_ratio.safe_sub(ONE_Q64)?.safe_div(bin_step_u128)?;

        Ok(delta_bin_id.safe_mul(2)?)
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

#[derive(Default, Debug)]
pub struct FeeMode {
    pub fees_on_input: bool,
    pub fees_on_token_a: bool,
    pub has_referral: bool,
}

impl FeeMode {
    pub fn get_fee_mode(
        collect_fee_mode: u8,
        trade_direction: TradeDirection,
        has_referral: bool,
    ) -> Result<FeeMode> {
        let collect_fee_mode = CollectFeeMode::try_from(collect_fee_mode)
            .map_err(|_| PoolError::InvalidCollectFeeMode)?;

        let (fees_on_input, fees_on_token_a) = match (collect_fee_mode, trade_direction) {
            // When collecting fees on output token
            (CollectFeeMode::BothToken, TradeDirection::AtoB) => (false, false),
            (CollectFeeMode::BothToken, TradeDirection::BtoA) => (false, true),

            // When collecting fees on tokenB
            (CollectFeeMode::OnlyB, TradeDirection::AtoB) => (false, false),
            (CollectFeeMode::OnlyB, TradeDirection::BtoA) => (true, false),
        };

        Ok(FeeMode {
            fees_on_input,
            fees_on_token_a,
            has_referral,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{params::swap::TradeDirection, state::CollectFeeMode};

    use super::*;

    #[test]
    fn test_fee_mode_output_token_a_to_b() {
        let fee_mode =
            FeeMode::get_fee_mode(CollectFeeMode::BothToken as u8, TradeDirection::AtoB, false)
                .unwrap();

        assert_eq!(fee_mode.fees_on_input, false);
        assert_eq!(fee_mode.fees_on_token_a, false);
        assert_eq!(fee_mode.has_referral, false);
    }

    #[test]
    fn test_fee_mode_output_token_b_to_a() {
        let fee_mode =
            FeeMode::get_fee_mode(CollectFeeMode::BothToken as u8, TradeDirection::BtoA, true)
                .unwrap();

        assert_eq!(fee_mode.fees_on_input, false);
        assert_eq!(fee_mode.fees_on_token_a, true);
        assert_eq!(fee_mode.has_referral, true);
    }

    #[test]
    fn test_fee_mode_quote_token_a_to_b() {
        let fee_mode =
            FeeMode::get_fee_mode(CollectFeeMode::OnlyB as u8, TradeDirection::AtoB, false)
                .unwrap();

        assert_eq!(fee_mode.fees_on_input, false);
        assert_eq!(fee_mode.fees_on_token_a, false);
        assert_eq!(fee_mode.has_referral, false);
    }

    #[test]
    fn test_fee_mode_quote_token_b_to_a() {
        let fee_mode =
            FeeMode::get_fee_mode(CollectFeeMode::OnlyB as u8, TradeDirection::BtoA, true).unwrap();

        assert_eq!(fee_mode.fees_on_input, true);
        assert_eq!(fee_mode.fees_on_token_a, false);
        assert_eq!(fee_mode.has_referral, true);
    }

    #[test]
    fn test_invalid_collect_fee_mode() {
        let result = FeeMode::get_fee_mode(
            2, // Invalid mode
            TradeDirection::BtoA,
            false,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_fee_mode_default() {
        let fee_mode = FeeMode::default();

        assert_eq!(fee_mode.fees_on_input, false);
        assert_eq!(fee_mode.fees_on_token_a, false);
        assert_eq!(fee_mode.has_referral, false);
    }

    // Property-based test to ensure consistent behavior
    #[test]
    fn test_fee_mode_properties() {
        // When trading BaseToQuote, fees should never be on input
        let fee_mode =
            FeeMode::get_fee_mode(CollectFeeMode::OnlyB as u8, TradeDirection::AtoB, true).unwrap();
        assert_eq!(fee_mode.fees_on_input, false);

        // When using QuoteToken mode, base_token should always be false
        let fee_mode =
            FeeMode::get_fee_mode(CollectFeeMode::OnlyB as u8, TradeDirection::BtoA, false)
                .unwrap();
        assert_eq!(fee_mode.fees_on_token_a, false);
    }
}
