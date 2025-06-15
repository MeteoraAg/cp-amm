use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

use crate::{constants::activation::*, math::safe_math::SafeMath, PoolError};

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    AnchorSerialize,
    AnchorDeserialize,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[repr(u8)]
/// Type of the activation
pub enum ActivationType {
    Slot,
    Timestamp,
}

pub struct ActivationHandler {
    /// current slot or current timestamp
    pub curr_point: u64,
    /// activation slot or activation timestamp
    pub activation_point: u64,
    /// buffer duration
    pub buffer_duration: u64,
    /// whitelisted vault
    pub whitelisted_vault: Pubkey,
}

impl ActivationHandler {
    pub fn get_current_point(activation_type: u8) -> Result<u64> {
        let activation_type = ActivationType::try_from(activation_type)
            .map_err(|_| PoolError::InvalidActivationType)?;
        let current_point = match activation_type {
            ActivationType::Slot => Clock::get()?.slot,
            ActivationType::Timestamp => Clock::get()?.unix_timestamp as u64,
        };
        Ok(current_point)
    }

    pub fn get_current_point_and_max_vesting_duration(activation_type: u8) -> Result<(u64, u64)> {
        let activation_type = ActivationType::try_from(activation_type)
            .map_err(|_| PoolError::InvalidActivationType)?;
        let (curr_point, max_vesting_duration) = match activation_type {
            ActivationType::Slot => (Clock::get()?.slot, MAX_VESTING_SLOT_DURATION),
            ActivationType::Timestamp => (
                Clock::get()?.unix_timestamp as u64,
                MAX_VESTING_TIME_DURATION,
            ),
        };
        Ok((curr_point, max_vesting_duration))
    }

    pub fn get_current_point_and_buffer_duration(activation_type: u8) -> Result<(u64, u64)> {
        let activation_type = ActivationType::try_from(activation_type)
            .map_err(|_| PoolError::InvalidActivationType)?;
        let (curr_point, buffer_duration) = match activation_type {
            ActivationType::Slot => (Clock::get()?.slot, SLOT_BUFFER),
            ActivationType::Timestamp => (Clock::get()?.unix_timestamp as u64, TIME_BUFFER),
        };
        Ok((curr_point, buffer_duration))
    }

    pub fn get_max_activation_point(activation_type: u8) -> Result<u64> {
        let activation_type = ActivationType::try_from(activation_type)
            .map_err(|_| PoolError::InvalidActivationType)?;
        let (curr_point, max_activation_duration) = match activation_type {
            ActivationType::Slot => (Clock::get()?.slot, MAX_ACTIVATION_SLOT_DURATION),
            ActivationType::Timestamp => (
                Clock::get()?.unix_timestamp as u64,
                MAX_ACTIVATION_TIME_DURATION,
            ),
        };
        Ok(curr_point.safe_add(max_activation_duration)?)
    }

    pub fn get_pre_activation_start_point(&self) -> Result<u64> {
        Ok(self.activation_point.safe_sub(self.buffer_duration)?)
    }

    /// last join pool from alpha-vault
    pub fn get_last_join_point(&self) -> Result<u64> {
        let pre_activation_start_point = self.get_pre_activation_start_point()?;
        let last_join_point =
            pre_activation_start_point.safe_sub(self.buffer_duration.safe_div(12)?)?; // 5 minutes
        Ok(last_join_point)
    }
}
