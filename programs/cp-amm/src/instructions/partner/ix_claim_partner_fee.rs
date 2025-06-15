use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    constants::seeds::POOL_AUTHORITY_PREFIX, state::Pool, token::transfer_from_pool,
    EvtClaimPartnerFee,
};

/// Accounts for partner to claim fees
#[event_cpi]
#[derive(Accounts)]
pub struct ClaimPartnerFeesCtx<'info> {
    /// CHECK: pool authority
    #[account(seeds = [POOL_AUTHORITY_PREFIX.as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        has_one = token_a_vault,
        has_one = token_b_vault,
        has_one = token_a_mint,
        has_one = token_b_mint,
        has_one = partner,
    )]
    pub pool: AccountLoader<'info, Pool>,

    /// The treasury token a account
    #[account(mut)]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The treasury token b account
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

    pub partner: Signer<'info>,

    /// Token a program
    pub token_a_program: Interface<'info, TokenInterface>,

    /// Token b program
    pub token_b_program: Interface<'info, TokenInterface>,
}

/// Partner claim fees.
pub fn handle_claim_partner_fee(
    ctx: Context<ClaimPartnerFeesCtx>,
    max_amount_a: u64,
    max_amount_b: u64,
) -> Result<()> {
    let mut pool = ctx.accounts.pool.load_mut()?;
    let (token_a_amount, token_b_amount) = pool.claim_partner_fee(max_amount_a, max_amount_b)?;

    if max_amount_a > 0 {
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

    if max_amount_b > 0 {
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

    emit_cpi!(EvtClaimPartnerFee {
        pool: ctx.accounts.pool.key(),
        token_a_amount,
        token_b_amount
    });
    Ok(())
}
