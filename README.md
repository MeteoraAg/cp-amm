
# Meteora Constant Product AMM (DAMM v2)

MCPA is a brand new AMM program of Meteora that includes almost all features from dynamic-amm v1 with new features:
- Fixed hot account issue from dynamic-amm v1, each pool includes a set of unique accounts for swap instruction (no shared accounts between 2 pools)
- Support for token2022. All token2022 with metadata pointer and transfer fee extensions are supported permissionlessly. Token mints with other extensions can be whitelisted by Meteora's admin
- Fee is not compounded on LP, which allows us to implement many cool features like: collecting fee only in one token (aka SOL), position NFT, creating permanent lock for position but still being able to claim fee
- Support for base fee scheduler and dynamic fee. In fee scheduler we support 2 modes: linear or exponential, while dynamic fee is based on volatility when users trade with the pool
- Support for a minimal version of concentrated liquidity, where the pool is constant-product but has a price range, allowing liquidity to be more concentrated, hence bringing more volume to pool

## Endpoints
### Admin
- create_config: create a static config key that includes all pre-defined parameters when user create pools with that config key.
- create_dynamic_config: create a dynamic config key that only define pool creator authority.
- create_token_badge: whitelist token mint, that has non-permissionless extensions (token2022)
- create_claim_fee_operator: whitelist an address to claim protocol fee
- close_claim_fee_operato: unwhitelist the address to claim protocol fee
- close_config: close a config key
- initialize_reward: initialize an on-chain liquidity mining for a pool
- update_reward_funder: update a whitelisted address to fund rewards for on-chain liquidity mining 
- update_reward_duration: update reward duration for liquidity mining
- set_pool_status: enable or disable pools. If pool is disabled, user can only be able to withdraw, can't add liquidity or swap

### Keeper to claim protocol fee
- claim_protocol_fee: claim protocol fee to Meteora's treasury address

### Token team (who run on-chain liquidity mining)
- fund_reward: fund reward for on-chain liquidity mining
- withdraw_ineligible_reward: withdraw ineligible reward 

### Partner (aka Launchpad)
- claim_partner_fee: claim partner fee

### Token deployer 
- initialize_pool: create a new pool from a static config key 
- initialize_pool_with_dynamic_config: create a new pool from a dynamic config key 
- initialize_customizable_pool: create a new pool with customizable parameters, should be only used by token deployer, that token can't be leaked.

### Liquidity provider
- create_position: create a new position nft, that holds liquidity that owner will deposit later
- add_liquidity: add liquidity to a pool 
- remove_liquidity: remove liquidity from a pool
- remove_all_liquidity: remove all liquidity from a pool
- claim_position_fee: claim position fee 
- lock_position: lock position with a vesting schedule
- refresh_vesting: refresh vesting schedule
- permanent_lock_position: lock position permanently 
- claim_reward: claim rewards from on-chain liquidity mining

### Trading bot/ user swap with pools
- swap: swap with the pool


## Config key state
- vault_config_key: alpha-vault address that is able to buy pool before activation_point
- pool_creator_authority: if this address is non-default, then only this address can create pool with that config key (for launchpad)
- pool_fees: includes base fee scheduler, dynamic-fee, protocol fee percent, partner fee percent, and referral fee percent configuration
- activation_type: determines whether pools are run in slot or timestamp 
- collect_fee_mode: determines whether pool should collect fees in both tokens or only one token
- sqrt_min_price: square root of min price for pools
- sqrt_max_price: square root of max price for pools

## Development

### Dependencies

- anchor 0.31.0
- solana 2.1.0
- rust 1.85.0

### Build

Program 

```
anchor build
```

CLI

```
cargo build -p cli
```

### Test

```
pnpm install
pnpm test
```

## Deployments

- Mainnet-beta: cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG
- Devnet: cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG

## Audits

The program has been audited. You can find the audit report [here](https://docs.meteora.ag/resources/audits#id-2.-damm-v2).
