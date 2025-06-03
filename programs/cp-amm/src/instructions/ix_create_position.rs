use anchor_lang::{
    prelude::*,
    solana_program,
    system_program::{create_account, CreateAccount},
};
use anchor_spl::{
    token::TokenAccount,
    token_2022::{
        self, initialize_account3, initialize_mint2,
        spl_token_2022::{
            self,
            extension::{metadata_pointer, ExtensionType},
        },
        InitializeAccount3, InitializeMint2, Token2022,
    },
    token_interface::{token_metadata_initialize, TokenMetadataInitialize},
};

use crate::{
    constants::seeds::{POOL_AUTHORITY_PREFIX, POSITION_NFT_ACCOUNT_PREFIX, POSITION_PREFIX},
    get_pool_access_validator,
    state::{Pool, Position, PositionType},
    token::update_account_lamports_to_minimum_balance,
    EvtCreatePosition, PoolError,
};

#[event_cpi]
#[derive(Accounts)]
pub struct CreatePositionCtx<'info> {
    /// CHECK: Receives the position NFT
    pub owner: UncheckedAccount<'info>,

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

    #[account(mut)]
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

    /// CHECK: pool authority
    #[account(seeds = [POOL_AUTHORITY_PREFIX.as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    /// Address paying to create the position. Can be anyone
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Program to create NFT mint/token account and transfer for token22 account
    pub token_2022_program: Program<'info, Token2022>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_position(ctx: Context<CreatePositionCtx>) -> Result<()> {
    {
        let pool = ctx.accounts.pool.load()?;
        let access_validator = get_pool_access_validator(&pool)?;
        require!(
            access_validator.can_create_position(),
            PoolError::PoolDisabled
        );
    }

    // init position
    let mut position = ctx.accounts.position.load_init()?;
    let mut pool = ctx.accounts.pool.load_mut()?;

    let liquidity = 0;

    position.initialize(
        &mut pool,
        ctx.accounts.pool.key(),
        ctx.accounts.position_nft_mint.key(),
        liquidity,
    )?;

    drop(position);
    // initialize position nft mint
    initialize_position_nft(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.position_nft_mint.to_account_info(),
        ctx.accounts.pool_authority.to_account_info(),
        ctx.accounts.position_nft_account.to_account_info(),
        ctx.accounts.pool.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.token_2022_program.to_account_info(),
        ctx.bumps.position_nft_account,
        pool.position_type,
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
        owner: ctx.accounts.owner.key(),
        position: ctx.accounts.position.key(),
        position_nft_mint: ctx.accounts.position_nft_mint.key(),
    });

    Ok(())
}

pub fn initialize_position_nft<'info>(
    payer: AccountInfo<'info>,
    position_nft_mint: AccountInfo<'info>,
    pool_authority: AccountInfo<'info>,
    position_nft_account: AccountInfo<'info>,
    pool: AccountInfo<'info>,
    owner: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    token2022_program: AccountInfo<'info>,
    position_nft_account_bump: u8,
    position_type: u8,
) -> Result<()> {
    let position_type =
        PositionType::try_from(position_type).map_err(|_| PoolError::TypeCastFailed)?;

    let mut extensions = vec![
        ExtensionType::MintCloseAuthority,
        ExtensionType::MetadataPointer,
    ];

    if position_type == PositionType::Immutable {
        extensions.push(ExtensionType::NonTransferable);
    }

    let space =
        ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&extensions)?;

    let lamports = Rent::get()?.minimum_balance(space);

    // create mint account
    create_account(
        CpiContext::new(
            system_program.clone(),
            CreateAccount {
                from: payer.clone(),
                to: position_nft_mint.clone(),
            },
        ),
        lamports,
        space as u64,
        token2022_program.key,
    )?;

    // initialize token extensions
    for e in extensions {
        match e {
            ExtensionType::MetadataPointer => {
                let ix = metadata_pointer::instruction::initialize(
                    token2022_program.key,
                    position_nft_mint.key,
                    Some(pool_authority.key()),
                    Some(position_nft_mint.key()),
                )?;
                solana_program::program::invoke(
                    &ix,
                    &[token2022_program.clone(), position_nft_mint.clone()],
                )?;
            }
            ExtensionType::MintCloseAuthority => {
                let ix = spl_token_2022::instruction::initialize_mint_close_authority(
                    token2022_program.key,
                    position_nft_mint.key,
                    Some(pool_authority.key),
                )?;
                solana_program::program::invoke(
                    &ix,
                    &[token2022_program.clone(), position_nft_mint.clone()],
                )?;
            }
            ExtensionType::NonTransferable => {
                let ix = spl_token_2022::instruction::initialize_non_transferable_mint(
                    token2022_program.key,
                    position_nft_account.key,
                )?;
                solana_program::program::invoke(
                    &ix,
                    &[token2022_program.clone(), position_nft_mint.clone()],
                )?;
            }
            _ => {
                return err!(PoolError::InvalidExtension);
            }
        }
    }

    // initialize mint account
    initialize_mint2(
        CpiContext::new(
            token2022_program.clone(),
            InitializeMint2 {
                mint: position_nft_mint.clone(),
            },
        ),
        0,
        pool_authority.key,
        Some(pool.key),
    )?;

    // create token account
    let position_nft_account_seeds =
        position_nft_account_seeds!(position_nft_mint.key, position_nft_account_bump);
    let space = TokenAccount::LEN;
    let lamports = Rent::get()?.minimum_balance(space);
    create_account(
        CpiContext::new_with_signer(
            system_program.clone(),
            CreateAccount {
                from: payer.clone(),
                to: position_nft_account.clone(),
            },
            &[&position_nft_account_seeds[..]],
        ),
        lamports,
        space as u64,
        token2022_program.key,
    )?;

    // create user position nft account
    initialize_account3(CpiContext::new_with_signer(
        token2022_program.clone(),
        InitializeAccount3 {
            account: position_nft_account.clone(),
            mint: position_nft_mint.clone(),
            authority: owner.clone(),
        },
        &[&position_nft_account_seeds[..]],
    ))?;

    Ok(())
}

pub fn create_position_nft<'info>(
    payer: AccountInfo<'info>,
    position_nft_mint: AccountInfo<'info>,
    pool_authority: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    position_nft_account: AccountInfo<'info>,
    pool_authority_bump: u8,
) -> Result<()> {
    // init token metadata
    let seeds = pool_authority_seeds!(pool_authority_bump);
    let signer_seeds = &[&seeds[..]];
    let cpi_accounts = TokenMetadataInitialize {
        program_id: token_program.clone(),
        mint: position_nft_mint.clone(),
        metadata: position_nft_mint.clone(),
        mint_authority: pool_authority.clone(),
        update_authority: pool_authority.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
    token_metadata_initialize(
        cpi_ctx,
        String::from("Meteora Position NFT"), // TODO do we need to allow user to input custom name?
        String::from("MPN"),
        String::from("https://raw.githubusercontent.com/MeteoraAg/token-metadata/main/meteora_position_nft.png"), // TODO update image
    )?;

    // transfer minimum rent to mint account
    update_account_lamports_to_minimum_balance(
        position_nft_mint.clone(),
        payer.clone(),
        system_program.clone(),
    )?;

    // Mint the NFT
    token_2022::mint_to(
        CpiContext::new_with_signer(
            token_program.clone(),
            token_2022::MintTo {
                mint: position_nft_mint.clone(),
                to: position_nft_account.clone(),
                authority: pool_authority.clone(),
            },
            &[&seeds[..]],
        ),
        1,
    )?;

    Ok(())
}
