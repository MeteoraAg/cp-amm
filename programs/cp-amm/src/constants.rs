use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::pubkey;

/// refer raydium clmm
pub const MIN_SQRT_PRICE: u128 = 4295048016;
/// refer raydium clmm
pub const MAX_SQRT_PRICE: u128 = 79226673521066979257578248091;

pub const LIQUIDITY_SCALE: u8 = 128;

pub const REWARD_RATE_SCALE: u8 = 64;

pub const TOTAL_REWARD_SCALE: u8 = 192;

pub const ONE_Q64: u128 = 1u128 << 64;

pub const BIN_STEP_BPS_DEFAULT: u16 = 1;

//  bin_step << 64 / BASIS_POINT_MAX
pub const BIN_STEP_BPS_U128_DEFAULT: u128 = 1844674407370955;

static_assertions::const_assert_eq!(LIQUIDITY_SCALE + REWARD_RATE_SCALE, TOTAL_REWARD_SCALE);

pub const BASIS_POINT_MAX: u64 = 10_000;

pub const U24_MAX: u32 = 0xffffff;

// Number of bits to scale. This will decide the position of the radix point.

// Number of rewards supported by pool
pub const NUM_REWARDS: usize = 2;

// Minimum reward duration
pub const MIN_REWARD_DURATION: u64 = 1;

pub const MAX_REWARD_DURATION: u64 = 31536000; // 1 year = 365 * 24 * 3600

pub const MAX_RATE_LIMITER_DURATION_IN_SECONDS: u64 = 60 * 60 * 12; // 12 hours
pub const MAX_RATE_LIMITER_DURATION_IN_SLOTS: u64 = 108000; // 12 hours

static_assertions::const_assert_eq!(
    MAX_RATE_LIMITER_DURATION_IN_SECONDS * 1000 / 400,
    MAX_RATE_LIMITER_DURATION_IN_SLOTS
);

pub mod activation {
    #[cfg(not(feature = "local"))]
    pub const SLOT_BUFFER: u64 = 9000; // 1 slot = 400 mls => 1 hour
    #[cfg(feature = "local")]
    pub const SLOT_BUFFER: u64 = 5;

    #[cfg(not(feature = "local"))]
    pub const TIME_BUFFER: u64 = 3600; // 1 hour
    #[cfg(feature = "local")]
    pub const TIME_BUFFER: u64 = 5; // 5 secs

    #[cfg(not(feature = "local"))]
    pub const MAX_ACTIVATION_SLOT_DURATION: u64 = SLOT_BUFFER * 24 * 31; // 31 days
    #[cfg(feature = "local")]
    pub const MAX_ACTIVATION_SLOT_DURATION: u64 = 30;

    #[cfg(not(feature = "local"))]
    pub const MAX_ACTIVATION_TIME_DURATION: u64 = TIME_BUFFER * 24 * 31; // 31 days
    #[cfg(feature = "local")]
    pub const MAX_ACTIVATION_TIME_DURATION: u64 = 30;

    pub const MAX_VESTING_SLOT_DURATION: u64 = SLOT_BUFFER * 24 * 365 * 10; // 10 years
    pub const MAX_VESTING_TIME_DURATION: u64 = TIME_BUFFER * 24 * 365 * 10; // 10 years

    pub const FIVE_MINUTES_SLOT_BUFFER: u64 = SLOT_BUFFER / 12; // 5 minutes

    pub const FIVE_MINUTES_TIME_BUFFER: u64 = TIME_BUFFER / 12; // 5 minutes

    pub const MAX_FEE_CURVE_TIME_DURATION: u64 = 3600 * 24; // 1 day
    pub const MAX_FEE_CURVE_SLOT_DURATION: u64 = 9000 * 24; // 1 day

    pub const MAX_HIGH_TAX_TIME_DURATION: u64 = TIME_BUFFER / 6; // 10 minutes
    pub const MAX_HIGH_TAX_SLOT_DURATION: u64 = SLOT_BUFFER / 6; // 10 minutes
}

/// Store constants related to fees
pub mod fee {

    /// Default fee denominator. DO NOT simply update it as it will break logic that depends on it as default value.
    pub const FEE_DENOMINATOR: u64 = 1_000_000_000;

    /// Max fee BPS
    pub const MAX_FEE_BPS_V0: u64 = 5000; // 50%
    pub const MAX_FEE_NUMERATOR_V0: u64 = 500_000_000; // 50%

    pub const MAX_FEE_BPS_V1: u64 = 9900; // 99%
    pub const MAX_FEE_NUMERATOR_V1: u64 = 990_000_000; // 99%

    /// Max basis point. 100% in pct
    pub const MAX_BASIS_POINT: u64 = 10000;

    pub const MIN_FEE_BPS: u64 = 1; // 0.01%
    pub const MIN_FEE_NUMERATOR: u64 = 100_000;

    static_assertions::const_assert_eq!(
        MAX_FEE_BPS_V0 * FEE_DENOMINATOR / MAX_BASIS_POINT,
        MAX_FEE_NUMERATOR_V0
    );

    static_assertions::const_assert_eq!(
        MAX_FEE_BPS_V1 * FEE_DENOMINATOR / MAX_BASIS_POINT,
        MAX_FEE_NUMERATOR_V1
    );

    static_assertions::const_assert_eq!(
        MIN_FEE_BPS * FEE_DENOMINATOR / MAX_BASIS_POINT,
        MIN_FEE_NUMERATOR
    );

    pub const CUSTOMIZABLE_PROTOCOL_FEE_PERCENT: u8 = 20; // 20%

    pub const CUSTOMIZABLE_HOST_FEE_PERCENT: u8 = 20; // 20%

    pub const MEME_MIN_FEE_UPDATE_WINDOW_DURATION: i64 = 60 * 30; // 30 minutes
}

pub mod seeds {
    pub const CONFIG_PREFIX: &[u8] = b"config";
    pub const CUSTOMIZABLE_POOL_PREFIX: &[u8] = b"cpool";
    pub const POOL_PREFIX: &[u8] = b"pool";
    pub const TOKEN_VAULT_PREFIX: &[u8] = b"token_vault";
    pub const POOL_AUTHORITY_PREFIX: &[u8] = b"pool_authority";
    pub const POSITION_PREFIX: &[u8] = b"position";
    pub const POSITION_NFT_ACCOUNT_PREFIX: &[u8] = b"position_nft_account";
    pub const TOKEN_BADGE_PREFIX: &[u8] = b"token_badge";
    pub const REWARD_VAULT_PREFIX: &[u8] = b"reward_vault";
    pub const CLAIM_FEE_OPERATOR_PREFIX: &[u8] = b"cf_operator";
}

pub mod treasury {
    use anchor_lang::{prelude::Pubkey, solana_program::pubkey};
    // https://app.squads.so/squads/4EWqcx3aNZmMetCnxwLYwyNjan6XLGp3Ca2W316vrSjv/treasury
    pub const ID: Pubkey = pubkey!("4EWqcx3aNZmMetCnxwLYwyNjan6XLGp3Ca2W316vrSjv");
}

// Supported quote mints
const SOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const DEFAULT_QUOTE_MINTS: [Pubkey; 2] = [SOL, USDC];
