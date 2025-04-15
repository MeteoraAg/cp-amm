use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::Cluster;
use clap::*;

#[derive(Parser, Debug)]
pub struct ConfigOverride {
    /// Cluster override
    ///
    /// Values = mainnet, testnet, devnet, localnet.
    /// Default: mainnet
    #[clap(global = true, long = "provider.cluster", default_value_t = Cluster::Localnet)]
    pub cluster: Cluster,
    /// Wallet override
    ///
    /// Example: /path/to/wallet/keypair.json
    /// Default: ~/.config/solana/id.json
    #[clap(
        global = true,
        long = "provider.wallet",
        default_value_t = String::from(shellexpand::tilde("~/.config/solana/id.json"))
    )]
    pub wallet: String,
    /// Priority fees in micro lamport. 1 lamport = 100_000 micro lamport
    #[clap(global = true, long = "priority-fee", default_value_t = 0)]
    pub priority_fee: u64,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Create a new config
    CreateConfig {
        #[clap(long)]
        sqrt_min_price: u128,

        #[clap(long)]
        sqrt_max_price: u128,

        #[clap(long)]
        vault_config_key: Pubkey,

        #[clap(long)]
        pool_creator_authority: Pubkey,

        #[clap(long)]
        activation_type: u8,

        #[clap(long)]
        collect_fee_mode: u8,

        #[clap(long)]
        trade_fee_numerator: u64,

        #[clap(long)]
        max_fee_bps: u64,

        #[clap(long)]
        protocol_fee_percent: u8,

        #[clap(long)]
        partner_fee_percent: u8,

        #[clap(long)]
        referral_fee_percent: u8,
    },
    /// Update config
    UpdateConfig {
        #[clap(long)]
        config: Pubkey,

        #[clap(long)]
        trade_fee_numerator: u64,

        #[clap(long)]
        max_fee_bps: u64,

        #[clap(long)]
        protocol_fee_percent: u8,

        #[clap(long)]
        partner_fee_percent: u8,

        #[clap(long)]
        referral_fee_percent: u8,
    },
    /// Close config
    CloseConfig {
        #[clap(long)]
        config: Pubkey,
    },
    /// create token badge
    CreateTokenBadge {
        #[clap(long)]
        token_mint: Pubkey,
    },
    // Create reward
    CreateReward {
        #[clap(long)]
        pool: Pubkey,
        #[clap(long)]
        reward_mint: Pubkey,
        #[clap(long)]
        reward_duration: u64,
    },
    // Fund reward
    FundReward {
        #[clap(long)]
        pool: Pubkey,

        #[clap(long)]
        reward_index: u8,

        #[clap(long)]
        carry_forward: bool,

        #[clap(long)]
        funding_amount: u64,
    },

    // update reward duration
    UpdateRewardDuration {
        #[clap(long)]
        pool: Pubkey,

        #[clap(long)]
        reward_index: u8,

        #[clap(long)]
        new_duration: u64,
    },
    // update reward funder
    UpdateRewardFunder {
        #[clap(long)]
        pool: Pubkey,

        #[clap(long)]
        reward_index: u8,

        #[clap(long)]
        new_funder: Pubkey,
    },
    // withdraw ineligible reward
    WithdrawIneligibleReward {
        #[clap(long)]
        pool: Pubkey,

        #[clap(long)]
        reward_index: u8,
    },
}

#[derive(Parser, Debug)]
#[clap(version, about, author)]
pub struct Cli {
    #[clap(flatten)]
    pub config_override: ConfigOverride,
    #[clap(subcommand)]
    pub command: Command,
}
