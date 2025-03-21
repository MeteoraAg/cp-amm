use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    activation_handler::ActivationHandler,
    constants::seeds::POOL_AUTHORITY_PREFIX,
    get_pool_access_validator,
    params::swap::TradeDirection,
    state::{CollectFeeMode, Pool},
    token::{
        calculate_transfer_fee_excluded_amount, calculate_transfer_fee_included_amount,
        transfer_from_pool, transfer_from_user,
    },
    EvtSwap, PoolError,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SwapParameters {
    amount: u64,
    threshold_amount: u64,
    is_swap_exact_in: bool,
}

#[event_cpi]
#[derive(Accounts)]
pub struct SwapCtx<'info> {
    /// CHECK: pool authority
    #[account(
        seeds = [
            POOL_AUTHORITY_PREFIX.as_ref(),
        ],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// Pool account
    #[account(mut, has_one = token_a_vault, has_one = token_b_vault)]
    pub pool: AccountLoader<'info, Pool>,

    /// The user token account for input token
    #[account(mut)]
    pub input_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token account for output token
    #[account(mut)]
    pub output_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for input token
    #[account(mut, token::token_program = token_a_program, token::mint = token_a_mint)]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(mut, token::token_program = token_b_program, token::mint = token_b_mint)]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token a
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token b
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The user performing the swap
    pub payer: Signer<'info>,

    /// Token a program
    pub token_a_program: Interface<'info, TokenInterface>,

    /// Token b program
    pub token_b_program: Interface<'info, TokenInterface>,

    /// referral token account
    #[account(mut)]
    pub referral_token_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
}

impl<'info> SwapCtx<'info> {
    /// Get the trading direction of the current swap. Eg: USDT -> USDC
    pub fn get_trade_direction(&self) -> TradeDirection {
        if self.input_token_account.mint == self.token_a_mint.key() {
            return TradeDirection::AtoB;
        }
        TradeDirection::BtoA
    }
}

pub fn handle_swap(ctx: Context<SwapCtx>, params: SwapParameters) -> Result<()> {
    {
        let pool = ctx.accounts.pool.load()?;
        let access_validator = get_pool_access_validator(&pool)?;
        require!(
            access_validator.can_swap(&ctx.accounts.payer.key()),
            PoolError::PoolDisabled
        );
    }

    let SwapParameters {
        amount,
        threshold_amount,
        is_swap_exact_in,
    } = params;

    let trade_direction = ctx.accounts.get_trade_direction();
    let (
        token_in_mint,
        token_out_mint,
        input_vault_account,
        output_vault_account,
        input_program,
        output_program,
    ) = match trade_direction {
        TradeDirection::AtoB => (
            &ctx.accounts.token_a_mint,
            &ctx.accounts.token_b_mint,
            &ctx.accounts.token_a_vault,
            &ctx.accounts.token_b_vault,
            &ctx.accounts.token_a_program,
            &ctx.accounts.token_b_program,
        ),
        TradeDirection::BtoA => (
            &ctx.accounts.token_b_mint,
            &ctx.accounts.token_a_mint,
            &ctx.accounts.token_b_vault,
            &ctx.accounts.token_a_vault,
            &ctx.accounts.token_b_program,
            &ctx.accounts.token_a_program,
        ),
    };

    let amount_specified = if is_swap_exact_in {
        calculate_transfer_fee_excluded_amount(&token_in_mint, amount)?.amount
    } else {
        calculate_transfer_fee_included_amount(&token_out_mint, amount)?.amount
    };

    require!(amount_specified > 0, PoolError::AmountIsZero);

    let is_referral = ctx.accounts.referral_token_account.is_some();

    let mut pool = ctx.accounts.pool.load_mut()?;

    // update for dynamic fee reference
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    pool.update_pre_swap(current_timestamp)?;

    let current_point = ActivationHandler::get_current_point(pool.activation_type)?;

    let (swap_result, input_amount_specified) = if is_swap_exact_in {
        let swap_exact_in_result = pool.get_swap_result_exact_in(
            amount_specified,
            is_referral,
            trade_direction,
            current_point,
        )?;

        let output_amount_excluded_fee = calculate_transfer_fee_excluded_amount(
            &token_in_mint,
            swap_exact_in_result.output_amount,
        )?
        .amount;

        // validate slippgae
        require!(
            output_amount_excluded_fee >= threshold_amount,
            PoolError::ExceededSlippage
        );

        (swap_exact_in_result, swap_exact_in_result.input_amount)
    } else {
        let swap_exact_out_result = pool.get_swap_result_exact_out(
            amount_specified,
            is_referral,
            trade_direction,
            current_point,
        )?;

        let input_amount_included_fee = calculate_transfer_fee_included_amount(
            &token_in_mint,
            swap_exact_out_result.input_amount,
        )?
        .amount;

        // validate slippgae
        require!(
            input_amount_included_fee <= threshold_amount,
            PoolError::ExceededSlippage
        );

        (swap_exact_out_result, input_amount_included_fee)
    };

    pool.apply_swap_result(&swap_result, trade_direction, current_timestamp)?;

    // send to reserve
    transfer_from_user(
        &ctx.accounts.payer,
        token_in_mint,
        &ctx.accounts.input_token_account,
        &input_vault_account,
        input_program,
        input_amount_specified,
    )?;
    // send to user
    transfer_from_pool(
        ctx.accounts.pool_authority.to_account_info(),
        &token_out_mint,
        &output_vault_account,
        &ctx.accounts.output_token_account,
        output_program,
        swap_result.output_amount,
        ctx.bumps.pool_authority,
    )?;
    // send to referral
    if is_referral {
        let collect_fee_mode = CollectFeeMode::try_from(pool.collect_fee_mode)
            .map_err(|_| PoolError::InvalidCollectFeeMode)?;

        if collect_fee_mode == CollectFeeMode::OnlyB || trade_direction == TradeDirection::AtoB {
            transfer_from_pool(
                ctx.accounts.pool_authority.to_account_info(),
                &ctx.accounts.token_b_mint,
                &ctx.accounts.token_b_vault,
                &ctx.accounts.referral_token_account.clone().unwrap(),
                &ctx.accounts.token_b_program,
                swap_result.referral_fee,
                ctx.bumps.pool_authority,
            )?;
        } else {
            transfer_from_pool(
                ctx.accounts.pool_authority.to_account_info(),
                &ctx.accounts.token_a_mint,
                &ctx.accounts.token_a_vault,
                &ctx.accounts.referral_token_account.clone().unwrap(),
                &ctx.accounts.token_a_program,
                swap_result.referral_fee,
                ctx.bumps.pool_authority,
            )?;
        }
    }

    emit_cpi!(EvtSwap {
        pool: ctx.accounts.pool.key(),
        trade_direction: trade_direction.into(),
        params,
        swap_result,
        is_referral,
        amount_specified,
        current_timestamp,
    });

    Ok(())
}
