use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    constants::{seeds::POOL_AUTHORITY_PREFIX, treasury},
    state::{ClaimFeeOperator, Pool},
    token::transfer_from_pool,
    EvtClaimProtocolFee,
};

/// Accounts for withdraw protocol fees
#[event_cpi]
#[derive(Accounts)]
pub struct ClaimProtocolFeesCtx<'info> {
    /// CHECK: pool authority
    #[account(seeds = [POOL_AUTHORITY_PREFIX.as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut, has_one = token_a_vault, has_one = token_b_vault, has_one = token_a_mint, has_one = token_b_mint)]
    pub pool: AccountLoader<'info, Pool>,

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

    /// The treasury token a account
    #[account(
        mut,
        associated_token::authority = treasury::ID,
        associated_token::mint = token_a_mint,
        associated_token::token_program = token_a_program,
    )]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The treasury token b account
    #[account(
        mut,
        associated_token::authority = treasury::ID,
        associated_token::mint = token_b_mint,
        associated_token::token_program = token_b_program,
    )]
    pub token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Claim fee operator
    #[account(has_one = operator)]
    pub claim_fee_operator: AccountLoader<'info, ClaimFeeOperator>,

    /// Operator
    pub operator: Signer<'info>,

    /// Token a program
    pub token_a_program: Interface<'info, TokenInterface>,

    /// Token b program
    pub token_b_program: Interface<'info, TokenInterface>,
}

/// Withdraw protocol fees. Permissionless.
pub fn handle_claim_protocol_fee(
    ctx: Context<ClaimProtocolFeesCtx>,
    max_amount_a: u64,
    max_amount_b: u64,
) -> Result<()> {
    let mut pool = ctx.accounts.pool.load_mut()?;

    let (token_a_amount, token_b_amount) = pool.claim_protocol_fee(max_amount_a, max_amount_b)?;

    if token_a_amount > 0 {
        transfer_from_pool(
            ctx.accounts.pool_authority.to_account_info(),
            &ctx.accounts.token_a_mint,
            &ctx.accounts.token_a_vault,
            &ctx.accounts.token_a_account,
            &ctx.accounts.token_a_program,
            token_a_amount,
            ctx.bumps.pool_authority,
        )?;
    }

    if token_b_amount > 0 {
        transfer_from_pool(
            ctx.accounts.pool_authority.to_account_info(),
            &ctx.accounts.token_b_mint,
            &ctx.accounts.token_b_vault,
            &ctx.accounts.token_b_account,
            &ctx.accounts.token_b_program,
            token_b_amount,
            ctx.bumps.pool_authority,
        )?;
    }

    emit_cpi!(EvtClaimProtocolFee {
        pool: ctx.accounts.pool.key(),
        token_a_amount,
        token_b_amount
    });

    Ok(())
}
