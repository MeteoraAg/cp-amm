use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use std::cmp::{max, min};

use crate::{
    activation_handler::ActivationHandler,
    constants::seeds::{
        POOL_AUTHORITY_PREFIX, POOL_PREFIX, POSITION_NFT_ACCOUNT_PREFIX, POSITION_PREFIX,
        TOKEN_VAULT_PREFIX,
    },
    create_position_nft,
    curve::get_initialize_amounts,
    initialize_position_nft,
    params::activation::ActivationParams,
    state::{Config, ConfigType, Pool, PoolType, Position, PositionType},
    token::{
        calculate_transfer_fee_included_amount, get_token_program_flags, is_supported_mint,
        validate_and_get_position_type, transfer_from_user,
    },
    EvtCreatePosition, EvtInitializePool, PoolError,
};

// To fix IDL generation: https://github.com/coral-xyz/anchor/issues/3209
pub fn max_key(left: &Pubkey, right: &Pubkey) -> [u8; 32] {
    max(left, right).to_bytes()
}

pub fn min_key(left: &Pubkey, right: &Pubkey) -> [u8; 32] {
    min(left, right).to_bytes()
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializePoolParameters {
    /// initialize liquidity
    pub liquidity: u128,
    /// The init price of the pool as a sqrt(token_b/token_a) Q64.64 value
    pub sqrt_price: u128,
    /// activation point
    pub activation_point: Option<u64>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct InitializePoolCtx<'info> {
    /// CHECK: Pool creator
    pub creator: UncheckedAccount<'info>,

    /// CHECK: Unique token mint address, initialize in program
    #[account(mut)]
    pub position_nft_mint: Signer<'info>,

    /// CHECK: position nft account
    #[account(
        mut,
        seeds = [POSITION_NFT_ACCOUNT_PREFIX.as_ref(), position_nft_mint.key().as_ref()],
        bump
    )]
    pub position_nft_account: UncheckedAccount<'info>,

    /// Address paying to create the pool. Can be anyone
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Which config the pool belongs to.
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

pub fn handle_initialize_pool<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, InitializePoolCtx<'info>>,
    params: InitializePoolParameters,
) -> Result<()> {
    let mut position_type: u8 = PositionType::Mutable.into();
    if !is_supported_mint(&ctx.accounts.token_a_mint)? {
        let token_badge = ctx
            .remaining_accounts
            .get(0)
            .ok_or(PoolError::InvalidTokenBadge)?;
        let position_type_a = validate_and_get_position_type(ctx.accounts.token_a_mint.key(), token_badge)?;

        position_type = std::cmp::max(position_type, position_type_a);
    }

    if !is_supported_mint(&ctx.accounts.token_b_mint)? {
        let token_badge = ctx
            .remaining_accounts
            .get(1)
            .ok_or(PoolError::InvalidTokenBadge)?;
        let position_type_b = validate_and_get_position_type(ctx.accounts.token_b_mint.key(), token_badge)?;

        position_type = std::cmp::max(position_type, position_type_b);
    }

    let InitializePoolParameters {
        liquidity,
        sqrt_price,
        activation_point,
    } = params;

    require!(liquidity > 0, PoolError::InvalidMinimumLiquidity);

    // init pool
    let config = ctx.accounts.config.load()?;

    require!(
        config.get_config_type()? == ConfigType::Static,
        PoolError::InvalidConfigType
    );

    require!(
        config.pool_creator_authority.eq(&Pubkey::default())
            || config.pool_creator_authority.eq(&ctx.accounts.payer.key()),
        PoolError::InvalidAuthorityToCreateThePool
    );

    let activation_params = ActivationParams {
        activation_point,
        activation_type: config.activation_type,
        has_alpha_vault: config.has_alpha_vault(),
    };
    activation_params.validate()?;

    let activation_point = activation_point.unwrap_or(ActivationHandler::get_current_point(
        config.activation_type,
    )?);

    require!(
        sqrt_price >= config.sqrt_min_price && sqrt_price <= config.sqrt_max_price,
        PoolError::InvalidPriceRange
    );

    let (token_a_amount, token_b_amount) = get_initialize_amounts(
        config.sqrt_min_price,
        config.sqrt_max_price,
        sqrt_price,
        liquidity,
    )?;

    require!(
        token_a_amount > 0 || token_b_amount > 0,
        PoolError::AmountIsZero
    );
    let mut pool = ctx.accounts.pool.load_init()?;

    let token_a_flag: u8 = get_token_program_flags(&ctx.accounts.token_a_mint).into();
    let token_b_flag: u8 = get_token_program_flags(&ctx.accounts.token_b_mint).into();
    let pool_type: u8 = PoolType::Permissionless.into();

    let alpha_vault = config.get_whitelisted_alpha_vault(ctx.accounts.pool.key());
    pool.initialize(
        config.pool_fees.to_pool_fees_struct(),
        ctx.accounts.token_a_mint.key(),
        ctx.accounts.token_b_mint.key(),
        ctx.accounts.token_a_vault.key(),
        ctx.accounts.token_b_vault.key(),
        alpha_vault,
        config.pool_creator_authority,
        config.sqrt_min_price,
        config.sqrt_max_price,
        sqrt_price,
        activation_point,
        config.activation_type,
        token_a_flag,
        token_b_flag,
        liquidity,
        config.collect_fee_mode,
        pool_type,
        position_type,
    );

    // init position
    let mut position = ctx.accounts.position.load_init()?;

    position.initialize(
        &mut pool,
        ctx.accounts.pool.key(),
        ctx.accounts.position_nft_mint.key(),
        liquidity,
    )?;

    // create position nft
    drop(position);
    // initialize position nft mint
    initialize_position_nft(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.position_nft_mint.to_account_info(),
        ctx.accounts.pool_authority.to_account_info(),
        ctx.accounts.position_nft_account.to_account_info(),
        ctx.accounts.pool.to_account_info(),
        ctx.accounts.creator.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.token_2022_program.to_account_info(),
        ctx.bumps.position_nft_account,
        position_type,
    )?;

    // create and mint position nft

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
        pool_fees: config.pool_fees.to_pool_fee_parameters(),
        creator: ctx.accounts.creator.key(),
        payer: ctx.accounts.payer.key(),
        activation_point,
        activation_type: config.activation_type,
        token_a_flag,
        token_b_flag,
        sqrt_price,
        liquidity,
        sqrt_min_price: config.sqrt_min_price,
        sqrt_max_price: config.sqrt_max_price,
        alpha_vault,
        collect_fee_mode: config.collect_fee_mode,
        token_a_amount,
        token_b_amount,
        total_amount_a,
        total_amount_b,
        pool_type,
    });

    Ok(())
}
