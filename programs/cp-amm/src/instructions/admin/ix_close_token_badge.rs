use crate::{assert_eq_admin, state::TokenBadge, PoolError};
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CloseTokenBadgeCtx<'info> {
    #[account(
        mut,
        close = rent_receiver
    )]
    pub token_badge: AccountLoader<'info, TokenBadge>,

    #[account(
        mut,
        constraint = assert_eq_admin(admin.key()) @ PoolError::InvalidAdmin,
    )]
    pub admin: Signer<'info>,

    /// CHECK: Account to receive closed account rental SOL
    #[account(mut)]
    pub rent_receiver: UncheckedAccount<'info>,
}
