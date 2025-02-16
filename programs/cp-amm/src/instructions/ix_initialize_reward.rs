use crate::assert_eq_admin;
use crate::constants::seeds::POOL_AUTHORITY_PREFIX;
use crate::constants::{ MAX_REWARD_DURATION, MIN_REWARD_DURATION, NUM_REWARDS };
use crate::error::PoolError;
use crate::event::EvtInitializeReward;
use crate::state::pool::Pool;
use crate::token::{ get_token_program_flags, is_supported_mint };
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{ Mint, TokenAccount, TokenInterface };

#[event_cpi]
#[derive(Accounts)]
#[instruction(reward_index: u64)]
pub struct InitializeReward<'info> {
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    /// CHECK: pool authority
    #[account(seeds = [POOL_AUTHORITY_PREFIX.as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(
        init,
        seeds = [pool.key().as_ref(), reward_index.to_le_bytes().as_ref()],
        bump,
        payer = admin,
        token::mint = reward_mint,
        token::authority = pool_authority
    )]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        constraint = assert_eq_admin(admin.key()) @ PoolError::InvalidAdmin,
    )]
    pub admin: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeReward<'info> {
    fn validate(&self, reward_index: usize, reward_duration: u64) -> Result<()> {
        let pool = self.pool.load()?;

        require!(reward_index < NUM_REWARDS, PoolError::InvalidRewardIndex);

        require!(
            reward_duration >= MIN_REWARD_DURATION && reward_duration <= MAX_REWARD_DURATION,
            PoolError::InvalidRewardDuration
        );

        let reward_info = &pool.reward_infos[reward_index];
        require!(!reward_info.initialized(), PoolError::RewardInitialized);

        Ok(())
    }
}

pub fn handle_initialize_reward(
    ctx: Context<InitializeReward>,
    index: u64,
    reward_duration: u64,
    funder: Pubkey
) -> Result<()> {

    // validate reward mint
    require!(is_supported_mint(&ctx.accounts.reward_mint)?, PoolError::RewardMintIsNotSupport);

    let reward_index: usize = index.try_into().map_err(|_| PoolError::TypeCastFailed)?;
    ctx.accounts.validate(reward_index, reward_duration)?;

    let mut pool = ctx.accounts.pool.load_mut()?;
    let reward_info = &mut pool.reward_infos[reward_index];

    reward_info.init_reward(
        ctx.accounts.reward_mint.key(),
        ctx.accounts.reward_vault.key(),
        funder,
        reward_duration,
        get_token_program_flags(&ctx.accounts.reward_mint).into()
    );

    emit_cpi!(EvtInitializeReward {
        pool: ctx.accounts.pool.key(),
        reward_mint: ctx.accounts.reward_mint.key(),
        funder,
        reward_duration,
        reward_index: index,
    });

    Ok(())
}
