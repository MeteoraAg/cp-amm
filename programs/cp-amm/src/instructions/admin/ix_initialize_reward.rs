use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    assert_eq_admin,
    constants::{
        seeds::{POOL_AUTHORITY_PREFIX, REWARD_VAULT_PREFIX},
        MAX_REWARD_DURATION, MIN_REWARD_DURATION, NUM_REWARDS,
    },
    error::PoolError,
    event::EvtInitializeReward,
    state::Pool,
    token::{get_token_program_flags, is_supported_mint, is_token_badge_initialized},
};

#[event_cpi]
#[derive(Accounts)]
#[instruction(reward_index: u8)]
pub struct InitializeRewardCtx<'info> {
    /// CHECK: pool authority
    #[account(seeds = [POOL_AUTHORITY_PREFIX.as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    #[account(
        init,
        seeds = [REWARD_VAULT_PREFIX.as_ref(), pool.key().as_ref(), reward_index.to_le_bytes().as_ref()],
        bump,
        payer = payer,
        token::mint = reward_mint,
        token::authority = pool_authority
    )]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    pub signer: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeRewardCtx<'info> {
    fn validate(&self, reward_index: usize, reward_duration: u64) -> Result<()> {
        let pool = self.pool.load()?;

        require!(reward_index < NUM_REWARDS, PoolError::InvalidRewardIndex);

        require!(
            reward_duration >= MIN_REWARD_DURATION && reward_duration <= MAX_REWARD_DURATION,
            PoolError::InvalidRewardDuration
        );

        let reward_info = &pool.reward_infos[reward_index];
        require!(!reward_info.initialized(), PoolError::RewardInitialized);

        if reward_index == 0 {
            // only pool creator is allowed to initialize reward with index 0
            require!(
                pool.creator == self.signer.key(),
                PoolError::InvalidPoolCreator
            );
        } else {
            require!(assert_eq_admin(self.signer.key()), PoolError::InvalidAdmin);
        }

        Ok(())
    }
}

pub fn handle_initialize_reward<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, InitializeRewardCtx<'info>>,
    reward_index: u8,
    reward_duration: u64,
    funder: Pubkey,
) -> Result<()> {
    if !is_supported_mint(&ctx.accounts.reward_mint)? {
        require!(
            is_token_badge_initialized(
                ctx.accounts.reward_mint.key(),
                ctx.remaining_accounts
                    .get(0)
                    .ok_or(PoolError::InvalidTokenBadge)?
            )?,
            PoolError::InvalidTokenBadge
        );
    }

    let index: usize = reward_index
        .try_into()
        .map_err(|_| PoolError::TypeCastFailed)?;

    ctx.accounts.validate(index, reward_duration)?;

    let mut pool = ctx.accounts.pool.load_mut()?;
    let reward_info = &mut pool.reward_infos[index];

    reward_info.init_reward(
        ctx.accounts.reward_mint.key(),
        ctx.accounts.reward_vault.key(),
        funder,
        reward_duration,
        get_token_program_flags(&ctx.accounts.reward_mint).into(),
    );

    emit_cpi!(EvtInitializeReward {
        pool: ctx.accounts.pool.key(),
        reward_mint: ctx.accounts.reward_mint.key(),
        funder,
        reward_duration,
        reward_index,
    });

    Ok(())
}
