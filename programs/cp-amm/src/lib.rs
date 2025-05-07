#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

#[macro_use]
pub mod macros;

pub mod instructions;
pub use instructions::*;
pub mod constants;
pub mod error;
pub mod state;
pub use error::*;
pub mod event;
pub use event::*;
pub mod utils;
pub use utils::*;
pub mod math;
pub use math::*;
pub mod curve;
pub mod tests;

pub mod pool_action_access;
pub use pool_action_access::*;

pub mod params;

declare_id!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

#[program]
pub mod cp_amm {
    use super::*;

    /// ADMIN FUNCTIONS /////

    // create static config
    pub fn create_config(
        ctx: Context<CreateConfigCtx>,
        index: u64,
        config_parameters: StaticConfigParameters,
    ) -> Result<()> {
        instructions::handle_create_static_config(ctx, index, config_parameters)
    }

    // create static config
    pub fn create_dynamic_config(
        ctx: Context<CreateConfigCtx>,
        index: u64,
        config_parameters: DynamicConfigParameters,
    ) -> Result<()> {
        instructions::handle_create_dynamic_config(ctx, index, config_parameters)
    }

    pub fn create_token_badge(ctx: Context<CreateTokenBadgeCtx>) -> Result<()> {
        instructions::handle_create_token_badge(ctx)
    }

    pub fn create_claim_fee_operator(ctx: Context<CreateClaimFeeOperatorCtx>) -> Result<()> {
        instructions::handle_create_claim_fee_operator(ctx)
    }

    pub fn close_claim_fee_operator(ctx: Context<CloseClaimFeeOperatorCtx>) -> Result<()> {
        instructions::handle_close_claim_fee_operator(ctx)
    }

    pub fn close_config(ctx: Context<CloseConfigCtx>) -> Result<()> {
        instructions::handle_close_config(ctx)
    }

    pub fn initialize_reward<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, InitializeRewardCtx<'info>>,
        reward_index: u8,
        reward_duration: u64,
        funder: Pubkey,
    ) -> Result<()> {
        instructions::handle_initialize_reward(ctx, reward_index, reward_duration, funder)
    }

    pub fn fund_reward(
        ctx: Context<FundRewardCtx>,
        reward_index: u8,
        amount: u64,
        carry_forward: bool,
    ) -> Result<()> {
        instructions::handle_fund_reward(ctx, reward_index, amount, carry_forward)
    }

    pub fn withdraw_ineligible_reward(
        ctx: Context<WithdrawIneligibleRewardCtx>,
        reward_index: u8,
    ) -> Result<()> {
        instructions::handle_withdraw_ineligible_reward(ctx, reward_index)
    }

    pub fn update_reward_funder(
        ctx: Context<UpdateRewardFunderCtx>,
        reward_index: u8,
        new_funder: Pubkey,
    ) -> Result<()> {
        instructions::handle_update_reward_funder(ctx, reward_index, new_funder)
    }

    pub fn update_reward_duration(
        ctx: Context<UpdateRewardDurationCtx>,
        reward_index: u8,
        new_duration: u64,
    ) -> Result<()> {
        instructions::handle_update_reward_duration(ctx, reward_index, new_duration)
    }

    pub fn set_pool_status(ctx: Context<SetPoolStatusCtx>, status: u8) -> Result<()> {
        instructions::handle_set_pool_status(ctx, status)
    }

    pub fn claim_protocol_fee(ctx: Context<ClaimProtocolFeesCtx>) -> Result<()> {
        instructions::handle_claim_protocol_fee(ctx)
    }

    pub fn claim_partner_fee(
        ctx: Context<ClaimPartnerFeesCtx>,
        max_amount_a: u64,
        max_amount_b: u64,
    ) -> Result<()> {
        instructions::handle_claim_partner_fee(ctx, max_amount_a, max_amount_b)
    }

    /// USER FUNCTIONS ////

    pub fn initialize_pool<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, InitializePoolCtx<'info>>,
        params: InitializePoolParameters,
    ) -> Result<()> {
        instructions::handle_initialize_pool(ctx, params)
    }

    pub fn initialize_pool_with_dynamic_config<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, InitializePoolWithDynamicConfigCtx<'info>>,
        params: InitializeCustomizablePoolParameters,
    ) -> Result<()> {
        instructions::handle_initialize_pool_with_dynamic_config(ctx, params)
    }

    pub fn initialize_customizable_pool<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, InitializeCustomizablePoolCtx<'info>>,
        params: InitializeCustomizablePoolParameters,
    ) -> Result<()> {
        instructions::handle_initialize_customizable_pool(ctx, params)
    }

    pub fn create_position(ctx: Context<CreatePositionCtx>) -> Result<()> {
        instructions::handle_create_position(ctx)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidityCtx>,
        params: AddLiquidityParameters,
    ) -> Result<()> {
        instructions::handle_add_liquidity(ctx, params)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidityCtx>,
        params: RemoveLiquidityParameters,
    ) -> Result<()> {
        instructions::handle_remove_liquidity(
            ctx,
            Some(params.liquidity_delta),
            params.token_a_amount_threshold,
            params.token_b_amount_threshold,
        )
    }

    pub fn remove_all_liquidity(
        ctx: Context<RemoveLiquidityCtx>,
        token_a_amount_threshold: u64,
        token_b_amount_threshold: u64,
    ) -> Result<()> {
        instructions::handle_remove_liquidity(
            ctx,
            None,
            token_a_amount_threshold,
            token_b_amount_threshold,
        )
    }

    pub fn close_position(ctx: Context<ClosePositionCtx>) -> Result<()> {
        instructions::handle_close_position(ctx)
    }

    pub fn swap(ctx: Context<SwapCtx>, params: SwapParameters) -> Result<()> {
        instructions::handle_swap(ctx, params)
    }

    pub fn claim_position_fee(ctx: Context<ClaimPositionFeeCtx>) -> Result<()> {
        instructions::handle_claim_position_fee(ctx)
    }

    pub fn lock_position(ctx: Context<LockPositionCtx>, params: VestingParameters) -> Result<()> {
        instructions::handle_lock_position(ctx, params)
    }

    pub fn refresh_vesting<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, RefreshVesting<'info>>,
    ) -> Result<()> {
        instructions::handle_refresh_vesting(ctx)
    }

    pub fn permanent_lock_position(
        ctx: Context<PermanentLockPositionCtx>,
        permanent_lock_liquidity: u128,
    ) -> Result<()> {
        instructions::handle_permanent_lock_position(ctx, permanent_lock_liquidity)
    }

    pub fn claim_reward(ctx: Context<ClaimRewardCtx>, reward_index: u8) -> Result<()> {
        instructions::handle_claim_reward(ctx, reward_index)
    }
}
