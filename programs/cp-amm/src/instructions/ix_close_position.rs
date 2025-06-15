use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{self, Token2022},
    token_interface::{Mint, TokenAccount},
};

use crate::{
    constants::seeds::POOL_AUTHORITY_PREFIX,
    state::{Pool, Position},
    EvtClosePosition, PoolError,
};

#[event_cpi]
#[derive(Accounts)]
pub struct ClosePositionCtx<'info> {
    /// position_nft_mint
    #[account(mut, address = position.load()?.nft_mint,)]
    pub position_nft_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The token account for nft
    #[account(
        mut,
        constraint = position_nft_account.mint == position.load()?.nft_mint,
        constraint = position_nft_account.amount == 1,
        token::authority = owner
    )]
    pub position_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    #[account(
        mut,
        has_one = pool,
        close = rent_receiver
    )]
    pub position: AccountLoader<'info, Position>,

    /// CHECK: pool authority
    #[account(seeds = [POOL_AUTHORITY_PREFIX.as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    /// CHECK: rent receiver
    #[account(mut)]
    pub rent_receiver: UncheckedAccount<'info>,

    /// Owner of position
    pub owner: Signer<'info>,

    /// Program to create NFT mint/token account and transfer for token22 account
    pub token_program: Program<'info, Token2022>,
}

pub fn handle_close_position(ctx: Context<ClosePositionCtx>) -> Result<()> {
    let position = ctx.accounts.position.load()?;
    require!(position.is_empty()?, PoolError::PositionIsNotEmpty);

    let mut pool = ctx.accounts.pool.load_mut()?;
    pool.metrics.reduce_position()?;

    // burn
    token_2022::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_2022::Burn {
                mint: ctx.accounts.position_nft_mint.to_account_info(),
                from: ctx.accounts.position_nft_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        1,
    )?;

    // close position_nft_account
    token_2022::close_account(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token_2022::CloseAccount {
            account: ctx.accounts.position_nft_account.to_account_info(),
            destination: ctx.accounts.rent_receiver.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        },
    ))?;

    // close position_nft_mint
    let signer_seeds = pool_authority_seeds!(ctx.bumps.pool_authority);
    token_2022::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token_2022::CloseAccount {
            account: ctx.accounts.position_nft_mint.to_account_info(),
            destination: ctx.accounts.rent_receiver.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        },
        &[&signer_seeds[..]],
    ))?;

    emit_cpi!(EvtClosePosition {
        pool: ctx.accounts.pool.key(),
        owner: ctx.accounts.owner.key(),
        position: ctx.accounts.position.key(),
        position_nft_mint: ctx.accounts.position_nft_mint.key(),
    });

    Ok(())
}
