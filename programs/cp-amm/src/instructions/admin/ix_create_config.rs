use crate::activation_handler::ActivationHandler;
use crate::assert_eq_admin;
use crate::constants::seeds::CONFIG_PREFIX;
use crate::constants::MAX_SQRT_PRICE;
use crate::constants::MIN_SQRT_PRICE;
use crate::event;
use crate::params::customizable_params::CustomizableParams;
use crate::params::pool_fees::PartnerInfo;
use crate::params::pool_fees::PoolFees;
use crate::state::config::Config;
use crate::state::CollectFeeMode;
use crate::PoolError;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ConfigParameters {
    pub pool_fees: PoolFees,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub vault_config_key: Pubkey,
    pub pool_creator_authority: Pubkey,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
    pub index: u64,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(config_parameters: ConfigParameters)]
pub struct CreateConfigCtx<'info> {
    #[account(
        init,
        seeds = [CONFIG_PREFIX.as_ref(), config_parameters.index.to_le_bytes().as_ref()],
        bump,
        payer = admin,
        space = 8 + Config::INIT_SPACE
    )]
    pub config: AccountLoader<'info, Config>,

    #[account(mut, constraint = assert_eq_admin(admin.key()) @ PoolError::InvalidAdmin)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_config(
    ctx: Context<CreateConfigCtx>,
    config_parameters: ConfigParameters
) -> Result<()> {
    let ConfigParameters {
        pool_fees,
        vault_config_key,
        pool_creator_authority,
        activation_type,
        sqrt_min_price,
        sqrt_max_price,
        collect_fee_mode,
        index,
    } = config_parameters;

    // Currently, only support price in fixed range: [MIN_SQRT_PRICE; MAX_SQRT_PRICE]
    require!(
        sqrt_min_price == MIN_SQRT_PRICE && sqrt_max_price == MAX_SQRT_PRICE,
        PoolError::InvalidPriceRange
    );

    // validate collect fee mode
    require!(CollectFeeMode::try_from(collect_fee_mode).is_ok(), PoolError::InvalidCollectFeeMode);

    // validate fee
    pool_fees.validate()?;

    let has_alpha_vault = vault_config_key.ne(&Pubkey::default());

    let activation_point = Some(ActivationHandler::get_max_activation_point(activation_type)?);

    let customizable_parameters = CustomizableParams {
        activation_point,
        has_alpha_vault,
        activation_type,
        trade_fee_numerator: pool_fees.trade_fee_numerator
            .try_into()
            .map_err(|_| PoolError::TypeCastFailed)?,
        padding: [0; 53],
    };

    // validate
    customizable_parameters.validate(&Clock::get()?)?;

    let partner_info = PartnerInfo {
        partner_authority: pool_creator_authority,
        fee_percent: pool_fees.partner_fee_percent,
        ..Default::default()
    };

    partner_info.validate()?;

    let mut config = ctx.accounts.config.load_init()?;
    config.init(
        index,
        &pool_fees,
        vault_config_key,
        pool_creator_authority,
        activation_type,
        sqrt_min_price,
        sqrt_max_price,
        collect_fee_mode.into()
    );

    emit_cpi!(event::EvtCreateConfig {
        pool_fees,
        config: ctx.accounts.config.key(),
        vault_config_key,
        pool_creator_authority,
        activation_type,
        index,
    });

    Ok(())
}
