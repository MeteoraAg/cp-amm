use std::ops::Deref;

use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::{solana_sdk::pubkey::Pubkey, solana_sdk::signer::Signer, Program};
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use anyhow::*;

use cp_amm::instruction;
use cp_amm::params::fee_parameters::{BaseFeeParameters, PoolFeeParameters};
use cp_amm::state::Config;
use cp_amm::{accounts, ConfigParameters};

use crate::common::pda::derive_event_authority_pda;

#[derive(Debug)]
pub struct UpdateConfigParams {
    pub config: Pubkey,
    pub base_fee: BaseFeeParameters,
    pub max_fee_bps: u64,
    pub protocol_fee_percent: u8,
    pub partner_fee_percent: u8,
    pub referral_fee_percent: u8,
}

pub fn update_config<C: Deref<Target = impl Signer> + Clone>(
    params: UpdateConfigParams,
    program: &Program<C>,
    transaction_config: RpcSendTransactionConfig,
    compute_unit_price: Option<Instruction>,
) -> Result<Pubkey> {
    let UpdateConfigParams {
        config,
        base_fee,
        max_fee_bps,
        protocol_fee_percent,
        partner_fee_percent,
        referral_fee_percent,
    } = params;

    let config_state = program.account::<Config>(config).unwrap();

    let event_authority = derive_event_authority_pda();
    let mut request_builder = program.request();
    // 1 close old config
    let close_config_data = (instruction::CloseConfig {}).data();
    let close_config_accounts = (accounts::CloseConfigCtx {
        config,
        admin: program.payer(),
        rent_receiver: program.payer(),
        event_authority,
        program: cp_amm::ID,
    })
    .to_account_metas(None);

    let close_config_ix = Instruction {
        data: close_config_data,
        accounts: close_config_accounts,
        program_id: cp_amm::ID,
    };

    // 2. create config
    let pool_fees = PoolFeeParameters {
        base_fee,
        protocol_fee_percent,
        partner_fee_percent,
        referral_fee_percent,
        max_fee_bps,
        dynamic_fee: None,
    };
    let config_parameters = ConfigParameters {
        pool_fees,
        vault_config_key: config_state.vault_config_key,
        pool_creator_authority: config_state.pool_creator_authority,
        activation_type: config_state.activation_type,
        sqrt_min_price: config_state.sqrt_min_price,
        sqrt_max_price: config_state.sqrt_max_price,
        collect_fee_mode: config_state.collect_fee_mode,
        index: config_state.index,
    };

    let create_config_data = (instruction::CreateConfig { config_parameters }).data();

    let create_config_accounts = (accounts::CreateConfigCtx {
        config,
        admin: program.payer(),
        system_program: anchor_client::solana_sdk::system_program::ID,
        event_authority,
        program: cp_amm::ID,
    })
    .to_account_metas(None);

    let create_config_ix = Instruction {
        data: create_config_data,
        accounts: create_config_accounts,
        program_id: cp_amm::ID,
    };

    //
    if let Some(compute_unit_price) = compute_unit_price {
        request_builder = request_builder.instruction(compute_unit_price);
    }

    request_builder = request_builder.instruction(close_config_ix);
    request_builder = request_builder.instruction(create_config_ix);

    let signature = request_builder.send_with_spinner_and_config(transaction_config);

    println!("Update config {config} Signature: {signature:#?}");

    signature?;

    Ok(config)
}
