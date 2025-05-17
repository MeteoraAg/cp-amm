use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    activation_handler::ActivationHandler,
    constants::seeds::{
        POOL_AUTHORITY_PREFIX, POOL_PREFIX, POSITION_NFT_ACCOUNT_PREFIX, POSITION_PREFIX,
        TOKEN_VAULT_PREFIX,
    },
    create_position_nft,
    curve::get_initialize_amounts,
    get_whitelisted_alpha_vault,
    state::{Config, ConfigType, Pool, PoolType, Position},
    token::{
        calculate_transfer_fee_included_amount, get_token_program_flags, is_supported_mint,
        is_token_badge_initialized, transfer_from_user,
    },
    EvtCreatePosition, EvtInitializePool, PoolError,
};

use super::{max_key, min_key, InitializeCustomizablePoolParameters};

#[event_cpi]
#[derive(Accounts)]
pub struct InitializePoolWithDynamicConfigCtx<'info> {
    /// CHECK: Pool creator
    pub creator: UncheckedAccount<'info>,

    /// position_nft_mint
    #[account(
        init,
        signer,
        payer = payer,
        mint::token_program = token_2022_program,
        mint::decimals = 0,
        mint::authority = pool_authority,
        mint::freeze_authority = pool, // use pool, so we can filter all position_nft_mint given pool address
        extensions::metadata_pointer::authority = pool_authority,
        extensions::metadata_pointer::metadata_address = position_nft_mint,
        extensions::close_authority::authority = pool_authority,
    )]
    pub position_nft_mint: Box<InterfaceAccount<'info, Mint>>,

    /// position nft account
    #[account(
        init,
        seeds = [POSITION_NFT_ACCOUNT_PREFIX.as_ref(), position_nft_mint.key().as_ref()],
        token::mint = position_nft_mint,
        token::authority = creator,
        token::token_program = token_2022_program,
        payer = payer,
        bump,
    )]
    pub position_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Address paying to create the pool. Can be anyone
    #[account(mut)]
    pub payer: Signer<'info>,

    pub pool_creator_authority: Signer<'info>,

    /// Which config the pool belongs to.
    #[account(has_one = pool_creator_authority)]
    pub config: AccountLoader<'info, Config>,

    /// CHECK: pool authority
    #[account(
        seeds = [
            POOL_AUTHORITY_PREFIX.as_ref(),
        ],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// Initialize an account to store the pool state
    #[account(
        init,
        seeds = [
            POOL_PREFIX.as_ref(),
            config.key().as_ref(),
            &max_key(&token_a_mint.key(), &token_b_mint.key()),
            &min_key(&token_a_mint.key(), &token_b_mint.key()),
        ],
        bump,
        payer = payer,
        space = 8 + Pool::INIT_SPACE
    )]
    pub pool: AccountLoader<'info, Pool>,

    #[account(
        init,
        seeds = [
            POSITION_PREFIX.as_ref(),
            position_nft_mint.key().as_ref()
        ],
        bump,
        payer = payer,
        space = 8 + Position::INIT_SPACE
    )]
    pub position: AccountLoader<'info, Position>,

    /// Token a mint
    #[account(
        constraint = token_a_mint.key() != token_b_mint.key(),
        mint::token_program = token_a_program,
    )]
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Token b mint
    #[account(
        mint::token_program = token_b_program,
    )]
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Token a vault for the pool
    #[account(
        init,
        seeds = [
            TOKEN_VAULT_PREFIX.as_ref(),
            token_a_mint.key().as_ref(),
            pool.key().as_ref(),
        ],
        token::mint = token_a_mint,
        token::authority = pool_authority,
        token::token_program = token_a_program,
        payer = payer,
        bump,
    )]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Token b vault for the pool
    #[account(
        init,
        seeds = [
            TOKEN_VAULT_PREFIX.as_ref(),
            token_b_mint.key().as_ref(),
            pool.key().as_ref(),
        ],
        token::mint = token_b_mint,
        token::authority = pool_authority,
        token::token_program = token_b_program,
        payer = payer,
        bump,
    )]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// payer token a account
    #[account(mut)]
    pub payer_token_a: Box<InterfaceAccount<'info, TokenAccount>>,

    /// creator token b account
    #[account(mut)]
    pub payer_token_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Program to create mint account and mint tokens
    pub token_a_program: Interface<'info, TokenInterface>,
    /// Program to create mint account and mint tokens
    pub token_b_program: Interface<'info, TokenInterface>,

    /// Program to create NFT mint/token account and transfer for token22 account
    pub token_2022_program: Program<'info, Token2022>,

    // Sysvar for program account
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_pool_with_dynamic_config<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, InitializePoolWithDynamicConfigCtx<'info>>,
    params: InitializeCustomizablePoolParameters,
) -> Result<()> {
    params.validate()?;
    if !is_supported_mint(&ctx.accounts.token_a_mint)? {
        require!(
            is_token_badge_initialized(
                ctx.accounts.token_a_mint.key(),
                ctx.remaining_accounts
                    .get(0)
                    .ok_or(PoolError::InvalidTokenBadge)?,
            )?,
            PoolError::InvalidTokenBadge
        )
    }

    if !is_supported_mint(&ctx.accounts.token_b_mint)? {
        require!(
            is_token_badge_initialized(
                ctx.accounts.token_b_mint.key(),
                ctx.remaining_accounts
                    .get(1)
                    .ok_or(PoolError::InvalidTokenBadge)?,
            )?,
            PoolError::InvalidTokenBadge
        )
    }

    let InitializeCustomizablePoolParameters {
        pool_fees,
        liquidity,
        sqrt_price,
        activation_point,
        sqrt_min_price,
        sqrt_max_price,
        activation_type,
        collect_fee_mode,
        has_alpha_vault,
    } = params;

    // init pool
    let config = ctx.accounts.config.load()?;

    require!(
        config.get_config_type()? == ConfigType::Dynamic,
        PoolError::InvalidConfigType
    );

    let (token_a_amount, token_b_amount) =
        get_initialize_amounts(sqrt_min_price, sqrt_max_price, sqrt_price, liquidity)?;
    require!(
        token_a_amount > 0 || token_b_amount > 0,
        PoolError::AmountIsZero
    );

    let mut pool = ctx.accounts.pool.load_init()?;

    let token_a_flag: u8 = get_token_program_flags(&ctx.accounts.token_a_mint).into();
    let token_b_flag: u8 = get_token_program_flags(&ctx.accounts.token_b_mint).into();
    let activation_point =
        activation_point.unwrap_or(ActivationHandler::get_current_point(activation_type)?);
    let alpha_vault = get_whitelisted_alpha_vault(
        ctx.accounts.payer.key(),
        ctx.accounts.pool.key(),
        has_alpha_vault,
    );
    let pool_type: u8 = PoolType::Customizable.into();
    pool.initialize(
        pool_fees.to_pool_fees_struct(),
        ctx.accounts.token_a_mint.key(),
        ctx.accounts.token_b_mint.key(),
        ctx.accounts.token_a_vault.key(),
        ctx.accounts.token_b_vault.key(),
        alpha_vault,
        config.pool_creator_authority,
        sqrt_min_price,
        sqrt_max_price,
        sqrt_price,
        activation_point,
        activation_type,
        token_a_flag,
        token_b_flag,
        liquidity,
        collect_fee_mode,
        pool_type,
    );

    let mut position = ctx.accounts.position.load_init()?;
    position.initialize(
        &mut pool,
        ctx.accounts.pool.key(),
        ctx.accounts.position_nft_mint.key(),
        liquidity,
    )?;

    // create position nft
    drop(position);
    create_position_nft(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.position_nft_mint.to_account_info(),
        ctx.accounts.pool_authority.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.token_2022_program.to_account_info(),
        ctx.accounts.position_nft_account.to_account_info(),
        ctx.bumps.pool_authority,
    )?;

    emit_cpi!(EvtCreatePosition {
        pool: ctx.accounts.pool.key(),
        owner: ctx.accounts.creator.key(),
        position: ctx.accounts.position.key(),
        position_nft_mint: ctx.accounts.position_nft_mint.key(),
    });

    // transfer token
    let total_amount_a =
        calculate_transfer_fee_included_amount(&ctx.accounts.token_a_mint, token_a_amount)?.amount;
    let total_amount_b =
        calculate_transfer_fee_included_amount(&ctx.accounts.token_b_mint, token_b_amount)?.amount;

    transfer_from_user(
        &ctx.accounts.payer,
        &ctx.accounts.token_a_mint,
        &ctx.accounts.payer_token_a,
        &ctx.accounts.token_a_vault,
        &ctx.accounts.token_a_program,
        total_amount_a,
    )?;
    transfer_from_user(
        &ctx.accounts.payer,
        &ctx.accounts.token_b_mint,
        &ctx.accounts.payer_token_b,
        &ctx.accounts.token_b_vault,
        &ctx.accounts.token_b_program,
        total_amount_b,
    )?;

    emit_cpi!(EvtInitializePool {
        pool: ctx.accounts.pool.key(),
        token_a_mint: ctx.accounts.token_a_mint.key(),
        token_b_mint: ctx.accounts.token_b_mint.key(),
        pool_fees,
        creator: ctx.accounts.creator.key(),
        payer: ctx.accounts.payer.key(),
        activation_point,
        activation_type,
        token_a_flag,
        token_b_flag,
        sqrt_price,
        liquidity,
        sqrt_min_price,
        sqrt_max_price,
        alpha_vault,
        collect_fee_mode,
        token_a_amount,
        token_b_amount,
        total_amount_a,
        total_amount_b,
        pool_type,
    });

    Ok(())
}
