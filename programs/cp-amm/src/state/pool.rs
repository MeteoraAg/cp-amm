use ruint::aliases::U256;
use static_assertions::const_assert_eq;
use std::cmp::min;
use std::u64;

use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    assert_eq_admin,
    constants::{LIQUIDITY_SCALE, NUM_REWARDS, REWARD_RATE_SCALE},
    curve::{
        get_delta_amount_a_unsigned, get_delta_amount_a_unsigned_unchecked,
        get_delta_amount_b_unsigned, get_next_sqrt_price_from_input,
        get_next_sqrt_price_from_output,
    },
    params::swap::TradeDirection,
    safe_math::SafeMath,
    state::{
        fee::{DynamicFeeStruct, FeeOnAmountResult, PoolFeesStruct},
        Position,
    },
    u128x128_math::{shl_div_256, Rounding},
    utils_math::{safe_mul_shr_cast, safe_shl_div_cast},
    PoolError,
};

use super::fee::FeeMode;

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
pub enum CollectFeeMode {
    /// Both token, in this mode only out token is collected
    BothToken,
    /// Only token B, we just need token B, because if user want to collect fee in token A, they just need to flip order of tokens
    OnlyB,
}

/// pool status
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
pub enum PoolStatus {
    Enable,
    Disable,
}

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
pub enum PoolType {
    Permissionless,
    Customizable,
}

#[account(zero_copy)]
#[derive(InitSpace, Debug, Default)]
pub struct Pool {
    /// Pool fee
    pub pool_fees: PoolFeesStruct,
    /// token a mint
    pub token_a_mint: Pubkey,
    /// token b mint
    pub token_b_mint: Pubkey,
    /// token a vault
    pub token_a_vault: Pubkey,
    /// token b vault
    pub token_b_vault: Pubkey,
    /// Whitelisted vault to be able to buy pool before activation_point
    pub whitelisted_vault: Pubkey,
    /// partner
    pub partner: Pubkey,
    /// liquidity share
    pub liquidity: u128,
    /// padding, previous reserve amount, be careful to use that field
    pub _padding: u128,
    /// protocol a fee
    pub protocol_a_fee: u64,
    /// protocol b fee
    pub protocol_b_fee: u64,
    /// partner a fee
    pub partner_a_fee: u64,
    /// partner b fee
    pub partner_b_fee: u64,
    /// min price
    pub sqrt_min_price: u128,
    /// max price
    pub sqrt_max_price: u128,
    /// current price
    pub sqrt_price: u128,
    /// Activation point, can be slot or timestamp
    pub activation_point: u64,
    /// Activation type, 0 means by slot, 1 means by timestamp
    pub activation_type: u8,
    /// pool status, 0: enable, 1 disable
    pub pool_status: u8,
    /// token a flag
    pub token_a_flag: u8,
    /// token b flag
    pub token_b_flag: u8,
    /// 0 is collect fee in both token, 1 only collect fee in token a, 2 only collect fee in token b
    pub collect_fee_mode: u8,
    /// pool type
    pub pool_type: u8,
    /// padding
    pub _padding_0: [u8; 2],
    /// cumulative
    pub fee_a_per_liquidity: [u8; 32], // U256
    /// cumulative
    pub fee_b_per_liquidity: [u8; 32], // U256
    // TODO: Is this large enough?
    pub permanent_lock_liquidity: u128,
    /// metrics
    pub metrics: PoolMetrics,
    /// Padding for further use
    pub _padding_1: [u64; 10],
    /// Farming reward information
    pub reward_infos: [RewardInfo; NUM_REWARDS],
}

const_assert_eq!(Pool::INIT_SPACE, 1104);

#[zero_copy]
#[derive(Debug, InitSpace, Default)]
pub struct PoolMetrics {
    pub total_lp_a_fee: u128,
    pub total_lp_b_fee: u128,
    pub total_protocol_a_fee: u64,
    pub total_protocol_b_fee: u64,
    pub total_partner_a_fee: u64,
    pub total_partner_b_fee: u64,
    pub total_position: u64,
    pub padding: u64,
}

const_assert_eq!(PoolMetrics::INIT_SPACE, 80);

impl PoolMetrics {
    pub fn inc_position(&mut self) -> Result<()> {
        self.total_position = self.total_position.safe_add(1)?;
        Ok(())
    }
    pub fn rec_position(&mut self) -> Result<()> {
        self.total_position = self.total_position.safe_sub(1)?;
        Ok(())
    }

    pub fn accumulate_fee(
        &mut self,
        lp_fee: u64,
        protocol_fee: u64,
        partner_fee: u64,
        is_token_a: bool,
    ) -> Result<()> {
        if is_token_a {
            self.total_lp_a_fee = self.total_lp_a_fee.safe_add(lp_fee.into())?;
            self.total_protocol_a_fee = self.total_protocol_a_fee.safe_add(protocol_fee)?;
            self.total_partner_a_fee = self.total_partner_a_fee.safe_add(partner_fee)?;
        } else {
            self.total_lp_b_fee = self.total_lp_b_fee.safe_add(lp_fee.into())?;
            self.total_protocol_b_fee = self.total_protocol_b_fee.safe_add(protocol_fee)?;
            self.total_partner_b_fee = self.total_partner_b_fee.safe_add(partner_fee)?;
        }

        Ok(())
    }
}

/// Stores the state relevant for tracking liquidity mining rewards
#[zero_copy]
#[derive(InitSpace, Default, Debug, PartialEq)]
pub struct RewardInfo {
    /// Indicates if the reward has been initialized
    pub initialized: u8,
    /// reward token flag
    pub reward_token_flag: u8,
    /// padding
    pub _padding_0: [u8; 6],
    /// Padding to ensure `reward_rate: u128` is 16-byte aligned
    pub _padding_1: [u8; 8], // 8 bytes
    /// Reward token mint.
    pub mint: Pubkey,
    /// Reward vault token account.
    pub vault: Pubkey,
    /// Authority account that allows to fund rewards
    pub funder: Pubkey,
    /// reward duration
    pub reward_duration: u64,
    /// reward duration end
    pub reward_duration_end: u64,
    /// reward rate
    pub reward_rate: u128,
    /// Reward per token stored
    pub reward_per_token_stored: [u8; 32], // U256
    /// The last time reward states were updated.
    pub last_update_time: u64,
    /// Accumulated seconds when the farm distributed rewards but the bin was empty.
    /// These rewards will be carried over to the next reward time window.
    pub cumulative_seconds_with_empty_liquidity_reward: u64,
}

const_assert_eq!(RewardInfo::INIT_SPACE, 192);

impl RewardInfo {
    /// Returns true if this reward is initialized.
    /// Once initialized, a reward cannot transition back to uninitialized.
    pub fn initialized(&self) -> bool {
        self.initialized != 0
    }

    pub fn is_valid_funder(&self, funder: Pubkey) -> bool {
        assert_eq_admin(funder) || funder.eq(&self.funder)
    }

    pub fn init_reward(
        &mut self,
        mint: Pubkey,
        vault: Pubkey,
        funder: Pubkey,
        reward_duration: u64,
        reward_token_flag: u8,
    ) {
        self.initialized = 1;
        self.mint = mint;
        self.vault = vault;
        self.funder = funder;
        self.reward_duration = reward_duration;
        self.reward_token_flag = reward_token_flag;
    }

    pub fn update_rewards(&mut self, liquidity_supply: u128, current_time: u64) -> Result<()> {
        // Update reward if it initialized
        if self.initialized() {
            if liquidity_supply > 0 {
                let reward_per_token_stored_delta = self
                    .calculate_reward_per_token_stored_since_last_update(
                        current_time,
                        liquidity_supply,
                    )?;

                self.accumulate_reward_per_token_stored(reward_per_token_stored_delta)?;
            } else {
                // Time period which the reward was distributed to empty
                let time_period = self.get_seconds_elapsed_since_last_update(current_time)?;

                // Save the time window of empty reward, and reward it in the next time window
                self.cumulative_seconds_with_empty_liquidity_reward = self
                    .cumulative_seconds_with_empty_liquidity_reward
                    .safe_add(time_period)?;
            }

            self.update_last_update_time(current_time);
        }

        Ok(())
    }

    pub fn update_last_update_time(&mut self, current_time: u64) {
        self.last_update_time = min(current_time, self.reward_duration_end);
    }

    pub fn get_seconds_elapsed_since_last_update(&self, current_time: u64) -> Result<u64> {
        let last_time_reward_applicable = min(current_time, self.reward_duration_end);
        let time_period = last_time_reward_applicable.safe_sub(self.last_update_time.into())?;

        Ok(time_period)
    }

    // To make it simple we truncate decimals of liquidity_supply for the calculation
    pub fn calculate_reward_per_token_stored_since_last_update(
        &self,
        current_time: u64,
        liquidity_supply: u128,
    ) -> Result<U256> {
        let time_period: u128 = self
            .get_seconds_elapsed_since_last_update(current_time)?
            .into();
        let total_reward = time_period.safe_mul(self.reward_rate.into())?;

        let reward_per_token_stored = shl_div_256(total_reward, liquidity_supply, LIQUIDITY_SCALE)
            .ok_or_else(|| PoolError::MathOverflow)?;
        Ok(reward_per_token_stored)
    }

    pub fn accumulate_reward_per_token_stored(&mut self, delta: U256) -> Result<()> {
        self.reward_per_token_stored = self
            .reward_per_token_stored()
            .safe_add(delta)?
            .to_le_bytes();
        Ok(())
    }

    pub fn reward_per_token_stored(&self) -> U256 {
        U256::from_le_bytes(self.reward_per_token_stored)
    }

    /// Farming rate after funding
    pub fn update_rate_after_funding(
        &mut self,
        current_time: u64,
        funding_amount: u64,
    ) -> Result<()> {
        let reward_duration_end = self.reward_duration_end;

        let total_amount = if current_time >= reward_duration_end {
            funding_amount
        } else {
            let remaining_seconds = reward_duration_end.safe_sub(current_time)?;
            let leftover: u64 = safe_mul_shr_cast(
                self.reward_rate,
                remaining_seconds.into(),
                REWARD_RATE_SCALE,
            )?;

            funding_amount.safe_add(leftover)?
        };

        self.reward_rate = safe_shl_div_cast(
            total_amount.into(),
            self.reward_duration.into(),
            REWARD_RATE_SCALE,
            Rounding::Down,
        )?;
        self.last_update_time = current_time;
        self.reward_duration_end = current_time.safe_add(self.reward_duration)?;

        Ok(())
    }
}

impl Pool {
    pub fn initialize(
        &mut self,
        pool_fees: PoolFeesStruct,
        token_a_mint: Pubkey,
        token_b_mint: Pubkey,
        token_a_vault: Pubkey,
        token_b_vault: Pubkey,
        whitelisted_vault: Pubkey,
        partner: Pubkey,
        sqrt_min_price: u128,
        sqrt_max_price: u128,
        sqrt_price: u128,
        activation_point: u64,
        activation_type: u8,
        token_a_flag: u8,
        token_b_flag: u8,
        liquidity: u128,
        collect_fee_mode: u8,
        pool_type: u8,
    ) {
        self.pool_fees = pool_fees;
        self.token_a_mint = token_a_mint;
        self.token_b_mint = token_b_mint;
        self.token_a_vault = token_a_vault;
        self.token_b_vault = token_b_vault;
        self.whitelisted_vault = whitelisted_vault;
        self.partner = partner;
        self.sqrt_min_price = sqrt_min_price;
        self.sqrt_max_price = sqrt_max_price;
        self.activation_point = activation_point;
        self.activation_type = activation_type;
        self.token_a_flag = token_a_flag;
        self.token_b_flag = token_b_flag;
        self.liquidity = liquidity;
        self.sqrt_price = sqrt_price;
        self.collect_fee_mode = collect_fee_mode;
        self.pool_type = pool_type;
    }

    pub fn pool_reward_initialized(&self) -> bool {
        self.reward_infos[0].initialized() || self.reward_infos[1].initialized()
    }

    pub fn get_swap_result(
        &self,
        amount: u64,
        fee_mode: &FeeMode,
        trade_direction: TradeDirection,
        current_point: u64,
        is_swap_exact_out: bool,
    ) -> Result<SwapResult> {
        let mut actual_protocol_fee = 0;
        let mut actual_lp_fee = 0;
        let mut actual_referral_fee = 0;
        let mut actual_partner_fee = 0;

        let actual_amount = if fee_mode.fees_on_input {
            let FeeOnAmountResult {
                amount: amount_after_fee,
                lp_fee,
                protocol_fee,
                partner_fee,
                referral_fee,
            } = self.pool_fees.get_fee_on_amount(
                amount,
                fee_mode.has_referral,
                current_point,
                self.activation_point,
                is_swap_exact_out,
            )?;

            actual_protocol_fee = protocol_fee;
            actual_lp_fee = lp_fee;
            actual_referral_fee = referral_fee;
            actual_partner_fee = partner_fee;

            amount_after_fee
        } else {
            amount
        };

        let SwapAmount {
            output_amount,
            next_sqrt_price,
        } = match trade_direction {
            TradeDirection::AtoB => {
                self.get_swap_result_from_a_to_b(actual_amount, is_swap_exact_out)
            }
            TradeDirection::BtoA => {
                self.get_swap_result_from_b_to_a(actual_amount, is_swap_exact_out)
            }
        }?;

        let actual_output_amount = if fee_mode.fees_on_input {
            output_amount
        } else {
            let FeeOnAmountResult {
                amount: amount_after_fee,
                lp_fee,
                protocol_fee,
                partner_fee,
                referral_fee,
            } = self.pool_fees.get_fee_on_amount(
                output_amount,
                fee_mode.has_referral,
                current_point,
                self.activation_point,
                is_swap_exact_out,
            )?;
            actual_protocol_fee = protocol_fee;
            actual_lp_fee = lp_fee;
            actual_referral_fee = referral_fee;
            actual_partner_fee = partner_fee;

            amount_after_fee
        };

        let (input_amount_result, output_amount_result) = if is_swap_exact_out {
            (actual_output_amount, amount)
        } else {
            (amount, actual_output_amount)
        };

        Ok(SwapResult {
            input_amount: input_amount_result,
            output_amount: output_amount_result,
            next_sqrt_price,
            lp_fee: actual_lp_fee,
            protocol_fee: actual_protocol_fee,
            partner_fee: actual_partner_fee,
            referral_fee: actual_referral_fee,
        })
    }

    fn get_swap_result_from_a_to_b(
        &self,
        amount: u64,
        is_swap_exact_out: bool,
    ) -> Result<SwapAmount> {
        // finding new target price
        let next_sqrt_price = if is_swap_exact_out {
            get_next_sqrt_price_from_output(self.sqrt_price, self.liquidity, amount, true)?
        } else {
            get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount, true)?
        };

        if next_sqrt_price < self.sqrt_min_price {
            return Err(PoolError::PriceRangeViolation.into());
        }

        // finding output amount
        let output_amount = if is_swap_exact_out {
            get_delta_amount_a_unsigned(
                next_sqrt_price,
                self.sqrt_price,
                self.liquidity,
                Rounding::Up,
            )?
        } else {
            get_delta_amount_b_unsigned(
                next_sqrt_price,
                self.sqrt_price,
                self.liquidity,
                Rounding::Down,
            )?
        };

        Ok(SwapAmount {
            output_amount,
            next_sqrt_price,
        })
    }

    fn get_swap_result_from_b_to_a(
        &self,
        amount: u64,
        is_swap_exact_out: bool,
    ) -> Result<SwapAmount> {
        // finding new target price
        let next_sqrt_price = if is_swap_exact_out {
            get_next_sqrt_price_from_output(self.sqrt_price, self.liquidity, amount, false)?
        } else {
            get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount, false)?
        };

        if next_sqrt_price > self.sqrt_max_price {
            return Err(PoolError::PriceRangeViolation.into());
        }
        // finding output amount
        let output_amount = if is_swap_exact_out {
            get_delta_amount_b_unsigned(
                self.sqrt_price,
                next_sqrt_price,
                self.liquidity,
                Rounding::Up,
            )?
        } else {
            get_delta_amount_a_unsigned(
                self.sqrt_price,
                next_sqrt_price,
                self.liquidity,
                Rounding::Down,
            )?
        };

        Ok(SwapAmount {
            output_amount,
            next_sqrt_price,
        })
    }

    pub fn apply_swap_result(
        &mut self,
        swap_result: &SwapResult,
        fee_mode: &FeeMode,
        current_timestamp: u64,
    ) -> Result<()> {
        let &SwapResult {
            input_amount: _input_amount,
            output_amount: _output_amount,
            lp_fee,
            next_sqrt_price,
            protocol_fee,
            partner_fee,
            referral_fee: _referral_fee,
        } = swap_result;

        let old_sqrt_price = self.sqrt_price;
        self.sqrt_price = next_sqrt_price;

        let fee_per_token_stored = shl_div_256(lp_fee.into(), self.liquidity, LIQUIDITY_SCALE)
            .ok_or_else(|| PoolError::MathOverflow)?;

        if fee_mode.fees_on_token_a {
            self.partner_a_fee = self.partner_a_fee.safe_add(partner_fee)?;
            self.protocol_a_fee = self.protocol_a_fee.safe_add(protocol_fee)?;
            self.fee_a_per_liquidity = self
                .fee_a_per_liquidity()
                .safe_add(fee_per_token_stored)?
                .to_le_bytes();
            self.metrics
                .accumulate_fee(lp_fee, protocol_fee, partner_fee, true)?;
        } else {
            self.partner_b_fee = self.partner_b_fee.safe_add(partner_fee)?;
            self.protocol_b_fee = self.protocol_b_fee.safe_add(protocol_fee)?;
            self.fee_b_per_liquidity = self
                .fee_b_per_liquidity()
                .safe_add(fee_per_token_stored)?
                .to_le_bytes();
            self.metrics
                .accumulate_fee(lp_fee, protocol_fee, partner_fee, false)?;
        }

        self.update_post_swap(old_sqrt_price, current_timestamp)?;

        Ok(())
    }

    pub fn get_amounts_for_modify_liquidity(
        &self,
        liquidity_delta: u128,
        round: Rounding,
    ) -> Result<ModifyLiquidityResult> {
        // finding output amount
        let token_a_amount = get_delta_amount_a_unsigned(
            self.sqrt_price,
            self.sqrt_max_price,
            liquidity_delta,
            round,
        )?;

        let token_b_amount = get_delta_amount_b_unsigned(
            self.sqrt_min_price,
            self.sqrt_price,
            liquidity_delta,
            round,
        )?;

        Ok(ModifyLiquidityResult {
            token_a_amount,
            token_b_amount,
        })
    }

    pub fn apply_add_liquidity(
        &mut self,
        position: &mut Position,
        liquidity_delta: u128,
    ) -> Result<()> {
        // update current fee for position
        position.update_fee(self.fee_a_per_liquidity(), self.fee_b_per_liquidity())?;

        // add liquidity
        position.add_liquidity(liquidity_delta)?;

        self.liquidity = self.liquidity.safe_add(liquidity_delta)?;

        Ok(())
    }

    pub fn apply_remove_liquidity(
        &mut self,
        position: &mut Position,
        liquidity_delta: u128,
    ) -> Result<()> {
        // update current fee for position
        position.update_fee(self.fee_a_per_liquidity(), self.fee_b_per_liquidity())?;

        // remove liquidity
        position.remove_unlocked_liquidity(liquidity_delta)?;

        self.liquidity = self.liquidity.safe_sub(liquidity_delta)?;

        Ok(())
    }

    pub fn get_max_amount_in(&self, trade_direction: TradeDirection) -> Result<u64> {
        let amount = match trade_direction {
            TradeDirection::AtoB => get_delta_amount_a_unsigned_unchecked(
                self.sqrt_min_price,
                self.sqrt_price,
                self.liquidity,
                Rounding::Down,
            )?,
            TradeDirection::BtoA => get_delta_amount_a_unsigned_unchecked(
                self.sqrt_price,
                self.sqrt_max_price,
                self.liquidity,
                Rounding::Down,
            )?,
        };
        if amount > U256::from(u64::MAX) {
            Ok(u64::MAX)
        } else {
            Ok(amount.try_into().unwrap())
        }
    }

    pub fn update_pre_swap(&mut self, current_timestamp: u64) -> Result<()> {
        if self.pool_fees.dynamic_fee.is_dynamic_fee_enable() {
            self.pool_fees
                .dynamic_fee
                .update_references(self.sqrt_price, current_timestamp)?;
        }
        Ok(())
    }

    pub fn update_post_swap(&mut self, old_sqrt_price: u128, current_timestamp: u64) -> Result<()> {
        if self.pool_fees.dynamic_fee.is_dynamic_fee_enable() {
            self.pool_fees
                .dynamic_fee
                .update_volatility_accumulator(self.sqrt_price)?;

            // update only last_update_timestamp if bin is crossed
            let delta_price = DynamicFeeStruct::get_delta_bin_id(
                self.pool_fees.dynamic_fee.bin_step_u128,
                old_sqrt_price,
                self.sqrt_price,
            )?;
            if delta_price > 0 {
                self.pool_fees.dynamic_fee.last_update_timestamp = current_timestamp;
            }
        }
        Ok(())
    }

    pub fn accumulate_permanent_locked_liquidity(
        &mut self,
        permanent_locked_liquidity: u128,
    ) -> Result<()> {
        self.permanent_lock_liquidity = self
            .permanent_lock_liquidity
            .safe_add(permanent_locked_liquidity)?;

        Ok(())
    }

    pub fn claim_protocol_fee(&mut self) -> (u64, u64) {
        let token_a_amount = self.protocol_a_fee;
        let token_b_amount = self.protocol_b_fee;
        self.protocol_a_fee = 0;
        self.protocol_b_fee = 0;
        (token_a_amount, token_b_amount)
    }

    pub fn claim_partner_fee(
        &mut self,
        max_amount_a: u64,
        max_amount_b: u64,
    ) -> Result<(u64, u64)> {
        let token_a_amount = self.partner_a_fee.min(max_amount_a);
        let token_b_amount = self.partner_b_fee.min(max_amount_b);
        self.partner_a_fee = self.partner_a_fee.safe_sub(token_a_amount)?;
        self.partner_b_fee = self.partner_b_fee.safe_sub(token_b_amount)?;
        Ok((token_a_amount, token_b_amount))
    }

    /// Update the rewards per token stored.
    pub fn update_rewards(&mut self, current_time: u64) -> Result<()> {
        for reward_idx in 0..NUM_REWARDS {
            let reward_info = &mut self.reward_infos[reward_idx];
            reward_info.update_rewards(self.liquidity, current_time)?;
        }

        Ok(())
    }

    pub fn claim_ineligible_reward(&mut self, reward_index: usize) -> Result<u64> {
        // calculate ineligible reward
        let reward_info = &mut self.reward_infos[reward_index];
        let ineligible_reward: u64 = safe_mul_shr_cast(
            reward_info
                .cumulative_seconds_with_empty_liquidity_reward
                .into(),
            reward_info.reward_rate,
            REWARD_RATE_SCALE,
        )?;

        reward_info.cumulative_seconds_with_empty_liquidity_reward = 0;

        Ok(ineligible_reward)
    }

    pub fn fee_a_per_liquidity(&self) -> U256 {
        U256::from_le_bytes(self.fee_a_per_liquidity)
    }

    pub fn fee_b_per_liquidity(&self) -> U256 {
        U256::from_le_bytes(self.fee_b_per_liquidity)
    }
}

/// Encodes all results of swapping
#[derive(Debug, PartialEq, AnchorDeserialize, AnchorSerialize)]
pub struct SwapResult {
    pub input_amount: u64,
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

pub struct SwapAmount {
    output_amount: u64,
    next_sqrt_price: u128,
}

#[derive(Debug, PartialEq)]
pub struct ModifyLiquidityResult {
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}
