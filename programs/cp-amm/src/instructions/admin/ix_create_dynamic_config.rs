use anchor_lang::prelude::*;

use crate::{event, PoolError};

use super::CreateConfigCtx;

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DynamicConfigParameters {
    pub pool_creator_authority: Pubkey,
}

pub fn handle_create_dynamic_config(
    ctx: Context<CreateConfigCtx>,
    index: u64,
    config_parameters: DynamicConfigParameters,
) -> Result<()> {
    let DynamicConfigParameters {
        pool_creator_authority,
    } = config_parameters;

    require!(
        pool_creator_authority.ne(&Pubkey::default()),
        PoolError::InvalidPoolCreatorAuthority
    );

    let mut config = ctx.accounts.config.load_init()?;
    config.init_dynamic_config(index, pool_creator_authority);

    emit_cpi!(event::EvtCreateDynamicConfig {
        config: ctx.accounts.config.key(),
        pool_creator_authority,
        index,
    });

    Ok(())
}
