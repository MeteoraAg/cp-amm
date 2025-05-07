use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use static_assertions::const_assert_eq;

use crate::{
    activation_handler::ActivationType,
    alpha_vault::alpha_vault,
    constants::activation::*,
    error::PoolError,
    params::fee_parameters::{
        BaseFeeParameters, DynamicFeeParameters, PartnerInfo, PoolFeeParameters,
    },
    safe_math::SafeMath,
    state::fee::{BaseFeeStruct, DynamicFeeStruct, PoolFeesStruct},
};

/// collect fee mode
#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    IntoPrimitive,
    TryFromPrimitive,
    AnchorDeserialize,
    AnchorSerialize,
    Default,
)]
pub enum ConfigType {
    /// In the static config type, initialized pool will take parameters from config state
    #[default]
    Static,
    /// In dynamic config type, pool creator can define customizable parameters, that mode is only available for private config
    Dynamic,
}

#[zero_copy]
#[derive(Debug, InitSpace, Default)]
pub struct PoolFeesConfig {
    pub base_fee: BaseFeeConfig,
    pub dynamic_fee: DynamicFeeConfig,
    pub protocol_fee_percent: u8,
    pub partner_fee_percent: u8,
    pub referral_fee_percent: u8,
    pub padding_0: [u8; 5],
    pub padding_1: [u64; 5],
}

const_assert_eq!(PoolFeesConfig::INIT_SPACE, 128);

#[zero_copy]
#[derive(Debug, InitSpace, Default)]
pub struct BaseFeeConfig {
    pub cliff_fee_numerator: u64,
    pub fee_scheduler_mode: u8,
    pub padding: [u8; 5],
    pub number_of_period: u16,
    pub period_frequency: u64,
    pub reduction_factor: u64,
}

const_assert_eq!(BaseFeeConfig::INIT_SPACE, 32);

impl BaseFeeConfig {
    fn to_base_fee_parameters(&self) -> BaseFeeParameters {
        BaseFeeParameters {
            cliff_fee_numerator: self.cliff_fee_numerator,
            number_of_period: self.number_of_period,
            period_frequency: self.period_frequency,
            reduction_factor: self.reduction_factor,
            fee_scheduler_mode: self.fee_scheduler_mode,
        }
    }

    fn to_base_fee_struct(&self) -> BaseFeeStruct {
        BaseFeeStruct {
            cliff_fee_numerator: self.cliff_fee_numerator,
            number_of_period: self.number_of_period,
            period_frequency: self.period_frequency,
            reduction_factor: self.reduction_factor,
            fee_scheduler_mode: self.fee_scheduler_mode,
            ..Default::default()
        }
    }
}

impl PoolFeesConfig {
    pub fn to_pool_fee_parameters(&self) -> PoolFeeParameters {
        let &PoolFeesConfig {
            base_fee,
            protocol_fee_percent,
            partner_fee_percent,
            referral_fee_percent,
            dynamic_fee:
                DynamicFeeConfig {
                    initialized,
                    bin_step,
                    bin_step_u128,
                    filter_period,
                    decay_period,
                    reduction_factor,
                    max_volatility_accumulator,
                    variable_fee_control,
                    ..
                },
            ..
        } = self;
        if initialized == 1 {
            PoolFeeParameters {
                base_fee: base_fee.to_base_fee_parameters(),
                protocol_fee_percent,
                partner_fee_percent,
                referral_fee_percent,
                dynamic_fee: Some(DynamicFeeParameters {
                    bin_step,
                    bin_step_u128,
                    filter_period,
                    decay_period,
                    reduction_factor,
                    max_volatility_accumulator,
                    variable_fee_control,
                }),
            }
        } else {
            PoolFeeParameters {
                base_fee: base_fee.to_base_fee_parameters(),
                protocol_fee_percent,
                partner_fee_percent,
                referral_fee_percent,
                ..Default::default()
            }
        }
    }

    pub fn to_pool_fees_struct(&self) -> PoolFeesStruct {
        let &PoolFeesConfig {
            base_fee,
            protocol_fee_percent,
            partner_fee_percent,
            referral_fee_percent,
            dynamic_fee,
            ..
        } = self;

        PoolFeesStruct {
            base_fee: base_fee.to_base_fee_struct(),
            protocol_fee_percent,
            partner_fee_percent,
            referral_fee_percent,
            dynamic_fee: dynamic_fee.to_dynamic_fee_struct(),
            ..Default::default()
        }
    }
}
#[zero_copy]
#[derive(Debug, InitSpace, Default)]
pub struct DynamicFeeConfig {
    pub initialized: u8, // 0, ignore for dynamic fee
    pub padding: [u8; 7],
    pub max_volatility_accumulator: u32,
    pub variable_fee_control: u32,
    pub bin_step: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub padding_1: [u8; 8], // Align to 16 bytes for `u128`
    pub bin_step_u128: u128,
}

const_assert_eq!(DynamicFeeConfig::INIT_SPACE, 48);

impl DynamicFeeConfig {
    fn to_dynamic_fee_struct(&self) -> DynamicFeeStruct {
        if self.initialized == 0 {
            DynamicFeeStruct::default()
        } else {
            DynamicFeeStruct {
                initialized: 1,
                bin_step: self.bin_step,
                bin_step_u128: self.bin_step_u128,
                filter_period: self.filter_period,
                decay_period: self.decay_period,
                reduction_factor: self.reduction_factor,
                max_volatility_accumulator: self.max_volatility_accumulator,
                variable_fee_control: self.variable_fee_control,
                ..Default::default()
            }
        }
    }
}

#[account(zero_copy)]
#[derive(InitSpace, Debug)]
pub struct Config {
    /// Vault config key
    pub vault_config_key: Pubkey,
    /// Only pool_creator_authority can use the current config to initialize new pool. When it's Pubkey::default, it's a public config.
    pub pool_creator_authority: Pubkey,
    /// Pool fee
    pub pool_fees: PoolFeesConfig,
    /// Activation type
    pub activation_type: u8,
    /// Collect fee mode
    pub collect_fee_mode: u8,
    /// Config type mode, 0 for static, 1 for dynamic
    pub config_type: u8,
    /// padding 0
    pub _padding_0: [u8; 5],
    /// config index
    pub index: u64,
    /// sqrt min price
    pub sqrt_min_price: u128,
    /// sqrt max price
    pub sqrt_max_price: u128,
    /// Fee curve point
    /// Padding for further use
    pub _padding_1: [u64; 10],
}

const_assert_eq!(Config::INIT_SPACE, 320);

pub struct BootstrappingConfig {
    pub activation_point: u64,
    pub vault_config_key: Pubkey,
    pub activation_type: u8,
}

pub struct TimingConstraint {
    pub current_point: u64,
    pub min_activation_duration: u64,
    pub max_activation_duration: u64,
    pub pre_activation_swap_duration: u64,
    pub last_join_buffer: u64,
    pub max_fee_curve_duration: u64,
    pub max_high_tax_duration: u64,
}

impl TimingConstraint {
    pub fn get_max_activation_point_from_current_time(&self) -> Result<u64> {
        Ok(self.current_point.safe_add(self.max_activation_duration)?)
    }
}

pub fn get_timing_constraint_by_activation_type(
    activation_type: ActivationType,
    clock: &Clock,
) -> TimingConstraint {
    match activation_type {
        ActivationType::Slot => TimingConstraint {
            current_point: clock.slot,
            min_activation_duration: SLOT_BUFFER,
            max_activation_duration: MAX_ACTIVATION_SLOT_DURATION,
            pre_activation_swap_duration: SLOT_BUFFER,
            last_join_buffer: FIVE_MINUTES_SLOT_BUFFER,
            max_fee_curve_duration: MAX_FEE_CURVE_SLOT_DURATION,
            max_high_tax_duration: MAX_HIGH_TAX_SLOT_DURATION,
        },
        ActivationType::Timestamp => TimingConstraint {
            current_point: clock.unix_timestamp as u64,
            min_activation_duration: TIME_BUFFER,
            max_activation_duration: MAX_ACTIVATION_TIME_DURATION,
            pre_activation_swap_duration: TIME_BUFFER,
            last_join_buffer: FIVE_MINUTES_TIME_BUFFER,
            max_fee_curve_duration: MAX_FEE_CURVE_TIME_DURATION,
            max_high_tax_duration: MAX_HIGH_TAX_TIME_DURATION,
        },
    }
}

impl Config {
    pub fn init_static_config(
        &mut self,
        index: u64,
        pool_fees: &PoolFeeParameters,
        vault_config_key: Pubkey,
        pool_creator_authority: Pubkey,
        activation_type: u8,
        sqrt_min_price: u128,
        sqrt_max_price: u128,
        collect_fee_mode: u8,
    ) {
        self.index = index;
        self.pool_fees = pool_fees.to_pool_fees_config();
        self.vault_config_key = vault_config_key;
        self.pool_creator_authority = pool_creator_authority;
        self.activation_type = activation_type;
        self.sqrt_min_price = sqrt_min_price;
        self.sqrt_max_price = sqrt_max_price;
        self.collect_fee_mode = collect_fee_mode;
        self.config_type = ConfigType::Static.into();
    }

    pub fn get_config_type(&self) -> Result<ConfigType> {
        let config_type =
            ConfigType::try_from(self.config_type).map_err(|_| PoolError::TypeCastFailed)?;
        Ok(config_type)
    }

    pub fn init_dynamic_config(&mut self, index: u64, pool_creator_authority: Pubkey) {
        self.index = index;
        self.pool_creator_authority = pool_creator_authority;
        self.config_type = ConfigType::Dynamic.into();
    }

    pub fn to_bootstrapping_config(&self, activation_point: u64) -> BootstrappingConfig {
        BootstrappingConfig {
            activation_point,
            vault_config_key: self.vault_config_key,
            activation_type: self.activation_type,
        }
    }

    pub fn get_partner_info(&self) -> PartnerInfo {
        PartnerInfo {
            partner_authority: self.pool_creator_authority,
            fee_percent: self.pool_fees.partner_fee_percent,
            ..Default::default()
        }
    }

    pub fn has_alpha_vault(&self) -> bool {
        self.vault_config_key.ne(&Pubkey::default())
    }

    pub fn get_whitelisted_alpha_vault(&self, pool: Pubkey) -> Pubkey {
        if self.vault_config_key.eq(&Pubkey::default()) {
            Pubkey::default()
        } else {
            alpha_vault::derive_vault_pubkey(self.vault_config_key, pool.key())
        }
    }

    pub fn get_max_activation_point_from_current_time(&self, clock: &Clock) -> Result<u64> {
        let timing_contraints = get_timing_constraint_by_activation_type(
            self.activation_type
                .try_into()
                .map_err(|_| PoolError::InvalidActivationType)?,
            clock,
        );
        timing_contraints.get_max_activation_point_from_current_time()
    }
}
