use std::u128;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    constants::seeds::POOL_AUTHORITY_PREFIX,
    get_pool_access_validator,
    state::{ModifyLiquidityResult, Pool, Position},
    token::{calculate_transfer_fee_excluded_amount, transfer_from_pool},
    u128x128_math::Rounding,
    EvtRemoveLiquidity, PoolError,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveLiquidityParameters {
    /// delta liquidity
    pub liquidity_delta: u128,
    /// minimum token a amount
    pub token_a_amount_threshold: u64,
    /// minimum token b amount
    pub token_b_amount_threshold: u64,
}

#[event_cpi]
#[derive(Accounts)]
pub struct RemoveLiquidityCtx<'info> {
    /// CHECK: pool authority
    #[account(seeds = [POOL_AUTHORITY_PREFIX.as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut, has_one = token_a_vault, has_one = token_b_vault, has_one = token_a_mint, has_one = token_b_mint)]
    pub pool: AccountLoader<'info, Pool>,

    #[account(
      mut,
      has_one = pool,
    )]
    pub position: AccountLoader<'info, Position>,

    /// The user token a account
    #[account(mut)]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token b account
    #[account(mut)]
    pub token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for input token
    #[account(mut, token::token_program = token_a_program, token::mint = token_a_mint)]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(mut, token::token_program = token_b_program, token::mint = token_b_mint)]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token a
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token b
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The token account for nft
    #[account(
            constraint = position_nft_account.mint == position.load()?.nft_mint,
            constraint = position_nft_account.amount == 1,
            token::authority = owner
    )]
    pub position_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// owner of position
    pub owner: Signer<'info>,

    /// Token a program
    pub token_a_program: Interface<'info, TokenInterface>,

    /// Token b program
    pub token_b_program: Interface<'info, TokenInterface>,
}

pub fn handle_remove_liquidity(
    ctx: Context<RemoveLiquidityCtx>,
    liquidity_delta: Option<u128>,
    token_a_amount_threshold: u64,
    token_b_amount_threshold: u64,
) -> Result<()> {
    {
        let pool = ctx.accounts.pool.load()?;
        let access_validator = get_pool_access_validator(&pool)?;
        require!(
            access_validator.can_remove_liquidity(),
            PoolError::PoolDisabled
        );
    }

    let mut pool = ctx.accounts.pool.load_mut()?;
    let mut position = ctx.accounts.position.load_mut()?;

    let liquidity_delta = liquidity_delta.unwrap_or(position.unlocked_liquidity);
    require!(
        liquidity_delta <= position.unlocked_liquidity && liquidity_delta > 0,
        PoolError::InsufficientLiquidity
    );

    // update current pool reward & postion reward before any logic
    let current_time = Clock::get()?.unix_timestamp as u64;
    position.update_rewards(&mut pool, current_time)?;

    let ModifyLiquidityResult {
        token_a_amount,
        token_b_amount,
    } = pool.get_amounts_for_modify_liquidity(liquidity_delta, Rounding::Down)?;

    require!(
        token_a_amount > 0 || token_b_amount > 0,
        PoolError::AmountIsZero
    );

    let transfer_fee_excluded_amount_a =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.token_a_mint, token_a_amount)?.amount;
    let transfer_fee_excluded_amount_b =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.token_b_mint, token_b_amount)?.amount;
    // Slippage check
    require!(
        transfer_fee_excluded_amount_a >= token_a_amount_threshold,
        PoolError::ExceededSlippage
    );
    require!(
        transfer_fee_excluded_amount_b >= token_b_amount_threshold,
        PoolError::ExceededSlippage
    );

    pool.apply_remove_liquidity(&mut position, liquidity_delta)?;

    // send to user
    transfer_from_pool(
        ctx.accounts.pool_authority.to_account_info(),
        &ctx.accounts.token_a_mint,
        &ctx.accounts.token_a_vault,
        &ctx.accounts.token_a_account,
        &ctx.accounts.token_a_program,
        token_a_amount,
        ctx.bumps.pool_authority,
    )?;
    transfer_from_pool(
        ctx.accounts.pool_authority.to_account_info(),
        &ctx.accounts.token_b_mint,
        &ctx.accounts.token_b_vault,
        &ctx.accounts.token_b_account,
        &ctx.accounts.token_b_program,
        token_b_amount,
        ctx.bumps.pool_authority,
    )?;

    emit_cpi!(EvtRemoveLiquidity {
        pool: ctx.accounts.pool.key(),
        owner: ctx.accounts.owner.key(),
        position: ctx.accounts.position.key(),
        params: RemoveLiquidityParameters {
            liquidity_delta,
            token_a_amount_threshold,
            token_b_amount_threshold
        },
        token_a_amount,
        token_b_amount,
    });

    Ok(())
}
