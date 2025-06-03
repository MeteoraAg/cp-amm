use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{
    assert_eq_admin,
    constants::seeds::TOKEN_BADGE_PREFIX,
    state::{PositionType, TokenBadge},
    token::is_supported_mint,
    EvtCreateTokenBadge, PoolError,
};

#[event_cpi]
#[derive(Accounts)]
pub struct CreateTokenBadgeCtx<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [
            TOKEN_BADGE_PREFIX.as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        space = 8 + TokenBadge::INIT_SPACE
    )]
    pub token_badge: AccountLoader<'info, TokenBadge>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = assert_eq_admin(admin.key()) @ PoolError::InvalidAdmin,
    )]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_token_badge(
    ctx: Context<CreateTokenBadgeCtx>,
    immutable_position_owner: u8,
) -> Result<()> {
    require!(
        !is_supported_mint(&ctx.accounts.token_mint)?,
        PoolError::CannotCreateTokenBadgeOnSupportedMint
    );
    // validate position type
    require!(
        PositionType::try_from(immutable_position_owner).is_ok(),
        PoolError::InvalidPositionType
    );

    let mut token_badge = ctx.accounts.token_badge.load_init()?;
    token_badge.initialize(ctx.accounts.token_mint.key(), immutable_position_owner)?;

    emit_cpi!(EvtCreateTokenBadge {
        token_mint: ctx.accounts.token_mint.key(),
    });

    Ok(())
}
