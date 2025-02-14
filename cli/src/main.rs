use std::rc::Rc;

use anchor_client::solana_sdk::compute_budget::ComputeBudgetInstruction;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::Client;
use anchor_client::{
    solana_client::rpc_config::RpcSendTransactionConfig,
    solana_sdk::{ commitment_config::CommitmentConfig, signer::{ keypair::*, Signer } },
};
use anyhow::*;
use clap::*;

#[cfg(feature = "e2e-test")]
mod tests;

use cp_amm::params::pool_fees::PoolFees;
use cp_amm::ConfigParameters;
use instructions::update_config::{
    update_config,
    update_pool_fee,
    UpdateConfigParams,
    UpdatePoolFeeParams,
};

mod instructions;
mod cmd;
mod common;

use crate::{
    cmd::{ Command, Cli },
    instructions::{
        create_config::create_config,
        close_config::{ close_config, CloseConfigParams },
        create_token_badge::create_token_badge,
    },
};

fn get_set_compute_unit_price_ix(micro_lamports: u64) -> Option<Instruction> {
    if micro_lamports > 0 {
        Some(ComputeBudgetInstruction::set_compute_unit_price(micro_lamports))
    } else {
        None
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let payer = read_keypair_file(cli.config_override.wallet).expect(
        "Wallet keypair file not found"
    );

    println!("Wallet {:#?}", payer.pubkey());

    let commitment_config = CommitmentConfig::finalized();
    let client = Client::new_with_options(
        cli.config_override.cluster,
        Rc::new(Keypair::from_bytes(&payer.to_bytes())?),
        commitment_config
    );

    let program = client.program(cp_amm::ID).unwrap();

    let transaction_config: RpcSendTransactionConfig = RpcSendTransactionConfig {
        skip_preflight: false,
        preflight_commitment: Some(commitment_config.commitment),
        encoding: None,
        max_retries: None,
        min_context_slot: None,
    };

    let compute_unit_price_ix = get_set_compute_unit_price_ix(cli.config_override.priority_fee);

    match cli.command {
        Command::CreateConfig {
            index,
            sqrt_min_price,
            sqrt_max_price,
            vault_config_key,
            pool_creator_authority,
            activation_type,
            collect_fee_mode,
            trade_fee_numerator,
            protocol_fee_percent,
            partner_fee_percent,
            referral_fee_percent,
        } => {
            let pool_fee = PoolFees {
                trade_fee_numerator,
                protocol_fee_percent,
                partner_fee_percent,
                referral_fee_percent,
                dynamic_fee: None, // TODO implement for dynamic fee
            };

            let params = ConfigParameters {
                index,
                sqrt_min_price,
                sqrt_max_price,
                vault_config_key,
                pool_creator_authority,
                activation_type,
                collect_fee_mode,
                pool_fees: pool_fee,
            };
            create_config(params, &program, transaction_config, compute_unit_price_ix)?;
        }
        Command::UpdateConfig { config, param, value } => {
            let params = UpdateConfigParams {
                config,
                param,
                value,
            };
            update_config(params, &program, transaction_config, compute_unit_price_ix)?;
        }

        Command::UpdatePoolFee { config, param, value } => {
            let params = UpdatePoolFeeParams {
                config,
                param,
                value,
            };
            update_pool_fee(params, &program, transaction_config, compute_unit_price_ix)?;
        }
        Command::CloseConfig { config, rent_receiver } => {
            let close_config_params = CloseConfigParams {
                config,
                rent_receiver,
            };
            close_config(close_config_params, &program, transaction_config, compute_unit_price_ix)?;
        }
        Command::CreateTokenBadge { token_mint } => {
            create_token_badge(token_mint, &program, transaction_config, compute_unit_price_ix)?;
        }
    }

    Ok(())
}
