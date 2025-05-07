//! Error module includes error messages and codes of the program
use anchor_lang::prelude::*;

/// Error messages and codes of the program
#[error_code]
#[derive(PartialEq)]
pub enum PoolError {
    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Invalid fee setup")]
    InvalidFee,

    #[msg("Exceeded slippage tolerance")]
    ExceededSlippage,

    #[msg("Pool disabled")]
    PoolDisabled,

    #[msg("Exceeded max fee bps")]
    ExceedMaxFeeBps,

    #[msg("Invalid admin")]
    InvalidAdmin,

    #[msg("Amount is zero")]
    AmountIsZero,

    #[msg("Type cast error")]
    TypeCastFailed,

    #[msg("Unable to modify activation point")]
    UnableToModifyActivationPoint,

    #[msg("Invalid authority to create the pool")]
    InvalidAuthorityToCreateThePool,

    #[msg("Invalid activation type")]
    InvalidActivationType,

    #[msg("Invalid activation point")]
    InvalidActivationPoint,

    #[msg("Quote token must be SOL,USDC")]
    InvalidQuoteMint,

    #[msg("Invalid fee curve")]
    InvalidFeeCurve,

    #[msg("Invalid Price Range")]
    InvalidPriceRange,

    #[msg("Trade is over price range")]
    PriceRangeViolation,

    #[msg("Invalid parameters")]
    InvalidParameters,

    #[msg("Invalid collect fee mode")]
    InvalidCollectFeeMode,

    #[msg("Invalid input")]
    InvalidInput,

    #[msg("Cannot create token badge on supported mint")]
    CannotCreateTokenBadgeOnSupportedMint,

    #[msg("Invalid token badge")]
    InvalidTokenBadge,

    #[msg("Invalid minimum liquidity")]
    InvalidMinimumLiquidity,

    #[msg("Invalid vesting information")]
    InvalidVestingInfo,

    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,

    #[msg("Invalid vesting account")]
    InvalidVestingAccount,

    #[msg("Invalid pool status")]
    InvalidPoolStatus,

    #[msg("Unsupported native mint token2022")]
    UnsupportNativeMintToken2022,

    #[msg("Invalid reward index")]
    InvalidRewardIndex,

    #[msg("Invalid reward duration")]
    InvalidRewardDuration,

    #[msg("Reward already initialized")]
    RewardInitialized,

    #[msg("Reward not initialized")]
    RewardUninitialized,

    #[msg("Invalid reward vault")]
    InvalidRewardVault,

    #[msg("Must withdraw ineligible reward")]
    MustWithdrawnIneligibleReward,

    #[msg("Reward duration is the same")]
    IdenticalRewardDuration,

    #[msg("Reward campaign in progress")]
    RewardCampaignInProgress,

    #[msg("Identical funder")]
    IdenticalFunder,

    #[msg("Invalid funder")]
    InvalidFunder,

    #[msg("Reward not ended")]
    RewardNotEnded,

    #[msg("Fee inverse is incorrect")]
    FeeInverseIsIncorrect,

    #[msg("Position is not empty")]
    PositionIsNotEmpty,

    #[msg("Invalid pool creator authority")]
    InvalidPoolCreatorAuthority,

    #[msg("Invalid config type")]
    InvalidConfigType,
}
