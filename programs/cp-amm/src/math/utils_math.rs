use super::{
    safe_math::SafeMath,
    u128x128_math::{mul_div, mul_shr, shl_div, Rounding},
};
use crate::error::PoolError;
use anchor_lang::prelude::Result;
use num_traits::cast::FromPrimitive;

/// safe_mul_div_cast
#[inline]
pub fn safe_mul_div_cast<T: FromPrimitive>(
    x: u128,
    y: u128,
    denominator: u128,
    rounding: Rounding,
) -> Result<T> {
    T::from_u128(mul_div(x, y, denominator, rounding).ok_or_else(|| PoolError::MathOverflow)?)
        .ok_or_else(|| PoolError::TypeCastFailed.into())
}

/// safe_mul_shr_cast
#[inline]
pub fn safe_mul_shr_cast<T: FromPrimitive>(
    x: u128,
    y: u128,
    offset: u8,
    rounding: Rounding,
) -> Result<T> {
    T::from_u128(mul_shr(x, y, offset, rounding).ok_or_else(|| PoolError::MathOverflow)?)
        .ok_or_else(|| PoolError::TypeCastFailed.into())
}

#[inline]
pub fn safe_mul_div_cast_u64<T: FromPrimitive>(x: u64, y: u64, denominator: u64) -> Result<T> {
    let result = u128::from(x)
        .safe_mul(y.into())?
        .safe_div(denominator.into())?;

    T::from_u128(result).ok_or_else(|| PoolError::TypeCastFailed.into())
}

#[inline]
pub fn safe_shl_div_cast<T: FromPrimitive>(
    x: u128,
    y: u128,
    offset: u8,
    rounding: Rounding,
) -> Result<T> {
    T::from_u128(shl_div(x, y, offset, rounding).ok_or_else(|| PoolError::MathOverflow)?)
        .ok_or_else(|| PoolError::TypeCastFailed.into())
}
