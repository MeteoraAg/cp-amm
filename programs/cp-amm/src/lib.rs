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

pub mod params;

#[cfg(feature = "local")]
declare_id!("9sh3gorJVsWgpdJo317PqnoWoTuDN2LkxiyYUUTu4sNJ");

#[cfg(not(feature = "local"))]
declare_id!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

#[program]
pub mod cp_amm {
    use super::*;

    /// Create config
    pub fn create_config(
        ctx: Context<CreateConfigCtx>,
        config_parameters: ConfigParameters,
    ) -> Result<()> {
        instructions::handle_create_config(ctx, config_parameters)
    }

    /// Create token badge
    pub fn create_token_badge(ctx: Context<CreateTokenBadgeCtx>) -> Result<()> {
        instructions::handle_create_token_badge(ctx)
    }

    /// Close config
    pub fn close_config(ctx: Context<CloseConfigCtx>) -> Result<()> {
        instructions::handle_close_config(ctx)
    }

    pub fn initialize_pool<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, InitializePoolCtx<'info>>,
        params: InitializePoolParameters,
    ) -> Result<()> {
        instructions::handle_initialize_pool(ctx, params)
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
        instructions::handle_remove_liquidity(ctx, params)
    }

    pub fn create_position(ctx: Context<CreatePositionCtx>) -> Result<()> {
        instructions::handle_create_position(ctx)
    }

    pub fn swap(ctx: Context<SwapCtx>, params: SwapParameters) -> Result<()> {
        instructions::handle_swap(ctx, params)
    }

    pub fn claim_position_fee(ctx: Context<ClaimPositionFeeCtx>) -> Result<()> {
        instructions::handle_claim_position_fee(ctx)
    }

    pub fn initialize_reward(
        ctx: Context<InitializeReward>,
        reward_index: u8,
        reward_duration: u64,
        funder: Pubkey
    ) -> Result<()> {
        instructions::handle_initialize_reward(ctx, reward_index, reward_duration, funder)
    }

    pub fn fund_reward(
        ctx: Context<FundReward>,
        reward_index: u64,
        amount: u64,
        carry_forward: bool
    ) -> Result<()> {
        instructions::handle_fund_reward(ctx, reward_index, amount, carry_forward)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>, reward_index: u64) -> Result<()> {
        instructions::handle_claim_reward(ctx, reward_index)
    }
}
