use anchor_lang::prelude::*;

use crate::{
    assert_eq_admin,
    constants::{MAX_REWARD_DURATION, MIN_REWARD_DURATION, NUM_REWARDS},
    state::Pool,
    EvtUpdateRewardDuration, PoolError,
};

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateRewardDurationCtx<'info> {
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    pub signer: Signer<'info>,
}

impl<'info> UpdateRewardDurationCtx<'info> {
    fn validate(&self, reward_index: usize, new_reward_duration: u64) -> Result<()> {
        require!(reward_index < NUM_REWARDS, PoolError::InvalidRewardIndex);

        require!(
            new_reward_duration >= MIN_REWARD_DURATION
                && new_reward_duration <= MAX_REWARD_DURATION,
            PoolError::InvalidRewardDuration
        );

        let pool = self.pool.load()?;
        let reward_info = &pool.reward_infos[reward_index];
        require!(reward_info.initialized(), PoolError::RewardInitialized);

        require!(
            reward_info.reward_duration != new_reward_duration,
            PoolError::IdenticalRewardDuration
        );

        let current_time = Clock::get()?.unix_timestamp;
        // only allow update reward duration if previous reward has been finished
        require!(
            reward_info.reward_duration_end < (current_time as u64),
            PoolError::RewardCampaignInProgress
        );

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

pub fn handle_update_reward_duration(
    ctx: Context<UpdateRewardDurationCtx>,
    reward_index: u8,
    new_reward_duration: u64,
) -> Result<()> {
    let index: usize = reward_index
        .try_into()
        .map_err(|_| PoolError::TypeCastFailed)?;

    ctx.accounts.validate(index, new_reward_duration)?;

    let mut pool = ctx.accounts.pool.load_mut()?;
    let reward_info = &mut pool.reward_infos[index];

    let old_reward_duration = reward_info.reward_duration;
    reward_info.reward_duration = new_reward_duration;

    emit_cpi!(EvtUpdateRewardDuration {
        pool: ctx.accounts.pool.key(),
        old_reward_duration,
        new_reward_duration,
        reward_index,
    });

    Ok(())
}
