use anchor_lang::prelude::*;

use crate::{
    activation_handler::ActivationHandler,
    assert_eq_admin,
    constants::{seeds::CONFIG_PREFIX, MAX_SQRT_PRICE, MIN_SQRT_PRICE},
    event,
    params::{
        activation::ActivationParams,
        fee_parameters::{PartnerInfo, PoolFeeParameters},
    },
    state::{CollectFeeMode, Config},
    PoolError,
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct StaticConfigParameters {
    pub pool_fees: PoolFeeParameters,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub vault_config_key: Pubkey,
    pub pool_creator_authority: Pubkey,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(index: u64)]
pub struct CreateConfigCtx<'info> {
    #[account(
        init,
        seeds = [CONFIG_PREFIX.as_ref(), index.to_le_bytes().as_ref()],
        bump,
        payer = admin,
        space = 8 + Config::INIT_SPACE
    )]
    pub config: AccountLoader<'info, Config>,

    #[account(mut, constraint = assert_eq_admin(admin.key()) @ PoolError::InvalidAdmin)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_static_config(
    ctx: Context<CreateConfigCtx>,
    index: u64,
    config_parameters: StaticConfigParameters,
) -> Result<()> {
    let StaticConfigParameters {
        pool_fees,
        vault_config_key,
        pool_creator_authority,
        activation_type,
        sqrt_min_price,
        sqrt_max_price,
        collect_fee_mode,
    } = config_parameters;

    require!(
        sqrt_min_price >= MIN_SQRT_PRICE && sqrt_max_price <= MAX_SQRT_PRICE,
        PoolError::InvalidPriceRange
    );
    // TODO do we need more buffer here?
    require!(
        sqrt_min_price < sqrt_max_price,
        PoolError::InvalidPriceRange
    );

    // validate collect fee mode
    require!(
        CollectFeeMode::try_from(collect_fee_mode).is_ok(),
        PoolError::InvalidCollectFeeMode
    );

    // validate fee
    pool_fees.validate()?;

    let has_alpha_vault = vault_config_key.ne(&Pubkey::default());

    let activation_point = Some(ActivationHandler::get_max_activation_point(
        activation_type,
    )?);

    let activation_params = ActivationParams {
        activation_point,
        activation_type,
        has_alpha_vault,
    };
    activation_params.validate()?;

    let partner_info = PartnerInfo {
        partner_authority: pool_creator_authority,
        fee_percent: pool_fees.partner_fee_percent,
        ..Default::default()
    };

    partner_info.validate()?;

    let mut config = ctx.accounts.config.load_init()?;
    config.init_static_config(
        index,
        &pool_fees,
        vault_config_key,
        pool_creator_authority,
        activation_type,
        sqrt_min_price,
        sqrt_max_price,
        collect_fee_mode.into(),
    );

    emit_cpi!(event::EvtCreateConfig {
        pool_fees,
        config: ctx.accounts.config.key(),
        vault_config_key,
        pool_creator_authority,
        activation_type,
        collect_fee_mode,
        sqrt_min_price,
        sqrt_max_price,
        index,
    });

    Ok(())
}
