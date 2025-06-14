#[cfg(test)]
pub const LIQUIDITY_MAX: u128 = 34028236692093846346337460743;

#[cfg(test)]
mod swap_tests;

#[cfg(test)]
mod modify_liquidity_tests;

#[cfg(test)]
mod overflow_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod dynamic_fee_tests;

#[cfg(test)]
mod price_math;

#[cfg(test)]
mod reward_tests;

#[cfg(test)]
mod fee_scheduler_tests;

#[cfg(test)]
mod test_volatility_accumulate;

#[cfg(test)]
mod test_rate_limiter;
