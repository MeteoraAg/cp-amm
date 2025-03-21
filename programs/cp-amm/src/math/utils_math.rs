use anchor_lang::prelude::Result;
use num_traits::cast::FromPrimitive;
use ruint::aliases::U256;

use crate::{
    safe_math::SafeMath,
    u128x128_math::{mul_shr, shl_div, Rounding},
    PoolError,
};

use super::u128x128_math::mul_shr_256;

/// safe_mul_shr_cast
#[inline]
pub fn safe_mul_shr_cast<T: FromPrimitive>(x: u128, y: u128, offset: u8) -> Result<T> {
    T::from_u128(mul_shr(x, y, offset).ok_or_else(|| PoolError::MathOverflow)?)
        .ok_or_else(|| PoolError::TypeCastFailed.into())
}

#[inline]
pub fn safe_mul_shr_256_cast<T: FromPrimitive>(x: U256, y: U256, offset: u8) -> Result<T> {
    T::from_u128(mul_shr_256(x, y, offset).ok_or_else(|| PoolError::MathOverflow)?)
        .ok_or_else(|| PoolError::TypeCastFailed.into())
}

#[inline]
pub fn safe_mul_div_cast_u64<T: FromPrimitive>(
    x: u64,
    y: u64,
    denominator: u64,
    rounding: Rounding,
) -> Result<T> {
    let prod = u128::from(x).safe_mul(y.into())?;

    let result = match rounding {
        // ceil(prod/denominator) is `(prod + denominator - 1) / denominator`
        Rounding::Up => {
            let ceil_prod: u128 = prod.safe_add(denominator.into())?.safe_sub(1u128)?;
            let result: u128 = ceil_prod.safe_div(denominator.into())?;
            result
        }
        Rounding::Down => prod.safe_div(denominator.into())?,
    };

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
