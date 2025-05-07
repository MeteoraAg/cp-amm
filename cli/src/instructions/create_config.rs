use std::ops::Deref;

use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::{solana_sdk::signer::Signer, Program};
use anchor_lang::prelude::Pubkey;
use anyhow::*;
use cp_amm::accounts;
use cp_amm::params::fee_parameters::PoolFeeParameters;
use cp_amm::{instruction, StaticConfigParameters};

use crate::common::pda::{derive_config_pda, derive_event_authority_pda};

pub struct CreateConfigParams {
    pub pool_fees: PoolFeeParameters,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub vault_config_key: Pubkey,
    pub pool_creator_authority: Pubkey,
    pub activation_type: u8,
    pub collect_fee_mode: u8,
}

pub fn create_config<C: Deref<Target = impl Signer> + Clone>(
    params: CreateConfigParams,
    program: &Program<C>,
    transaction_config: RpcSendTransactionConfig,
    compute_unit_price: Option<Instruction>,
) -> Result<()> {
    let mut index = 0u64;
    let CreateConfigParams {
        pool_fees,
        vault_config_key,
        pool_creator_authority,
        activation_type,
        sqrt_min_price,
        sqrt_max_price,
        collect_fee_mode,
    } = params;

    loop {
        let config = derive_config_pda(index);

        if program.rpc().get_account_data(&config).is_ok() {
            index += 1;
        } else {
            let event_authority = derive_event_authority_pda();

            let accounts = accounts::CreateConfigCtx {
                config,
                admin: program.payer(),
                system_program: anchor_client::solana_sdk::system_program::ID,
                event_authority,
                program: cp_amm::ID,
            };

            let config_parameters = StaticConfigParameters {
                pool_fees,
                vault_config_key,
                pool_creator_authority,
                activation_type,
                sqrt_min_price,
                sqrt_max_price,
                collect_fee_mode,
            };

            let ix = instruction::CreateConfig {
                index,
                config_parameters,
            };

            let mut request_builder = program.request();

            if let Some(compute_unit_price) = compute_unit_price {
                request_builder = request_builder.instruction(compute_unit_price);
            }

            let signature = request_builder
                .accounts(accounts)
                .args(ix)
                .send_with_spinner_and_config(transaction_config);

            println!("Initialize config {config} Signature: {signature:#?}");

            signature?;

            break;
        }
    }

    Ok(())
}
