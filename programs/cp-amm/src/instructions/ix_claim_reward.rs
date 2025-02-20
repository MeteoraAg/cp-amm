use crate::{
    constants::{ seeds::POOL_AUTHORITY_PREFIX, NUM_REWARDS },
    error::PoolError,
    event::EvtClaimReward,
    state::{ pool::Pool, position::Position },
    token::transfer_from_pool,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{ Mint, TokenAccount, TokenInterface };

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimRewardCtx<'info> {
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    /// CHECK: pool authority
    #[account(seeds = [POOL_AUTHORITY_PREFIX.as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        has_one = pool,
        has_one = owner,
    )]
    pub position: AccountLoader<'info, Position>,

    pub owner: Signer<'info>,

    #[account(mut)]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> ClaimRewardCtx<'info> {
    fn validate(&self, reward_index: usize) -> Result<()> {
        let pool = self.pool.load()?;
        require!(reward_index < NUM_REWARDS, PoolError::InvalidRewardIndex);

        let reward_info = &pool.reward_infos[reward_index];
        require!(reward_info.initialized(), PoolError::RewardUninitialized);
        require!(reward_info.vault.eq(&self.reward_vault.key()), PoolError::InvalidRewardVault);

        Ok(())
    }
}

pub fn handle_claim_reward(ctx: Context<ClaimRewardCtx>, index: u8) -> Result<()> {
    let reward_index: usize = index.try_into().map_err(|_| PoolError::TypeCastFailed)?;
    ctx.accounts.validate(reward_index)?;

    let mut position = ctx.accounts.position.load_mut()?;

    let mut pool = ctx.accounts.pool.load_mut()?;
    let current_time = Clock::get()?.unix_timestamp as u64;

    // update pool reward & position reward
    position.update_reward(&mut pool, current_time)?;

    // get all pending reward
    let total_reward = position.claim_reward(reward_index)?;

    // transfer rewards to user
    if total_reward > 0 {
        transfer_from_pool(
            ctx.accounts.pool_authority.to_account_info(),
            &ctx.accounts.reward_mint,
            &ctx.accounts.reward_vault,
            &ctx.accounts.user_token_account,
            &ctx.accounts.token_program,
            total_reward,
            ctx.bumps.pool_authority
        )?;
    }

    emit_cpi!(EvtClaimReward {
        pool: ctx.accounts.pool.key(),
        position: ctx.accounts.position.key(),
        mint_reward: ctx.accounts.reward_mint.key(),
        owner: position.owner,
        reward_index: index,
        total_reward,
    });

    Ok(())
}
