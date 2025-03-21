import { BN } from "@coral-xyz/anchor";

const SCALE_OFFSET = 64;
// Δa = L * (1 / √P_lower - 1 / √P_upper)
//
// Δa = L * (√P_upper - √P_lower) / (√P_upper * √P_lower)
//
// L = Δa * √P_upper * √P_lower / (√P_upper - √P_lower)
//
export function getLiquidityDeltaFromAmountA(
  maxAmountA: BN,
  lowerSqrtPrice: BN, // current sqrt price
  upperSqrtPrice: BN // max sqrt price
): BN {
  const prod = maxAmountA.mul(upperSqrtPrice.mul(lowerSqrtPrice));
  const delta = upperSqrtPrice.sub(lowerSqrtPrice);

  return prod.div(delta);
}

// Δb = L (√P_upper - √P_lower)
// L = Δb / (√P_upper - √P_lower)
export function getLiquidityDeltaFromAmountB(
  maxAmountB: BN,
  lowerSqrtPrice: BN, // min sqrt price
  upperSqrtPrice: BN // current sqrt price
): BN {
  const denominator = upperSqrtPrice.sub(lowerSqrtPrice);
  const result = maxAmountB.shln(SCALE_OFFSET * 2).div(denominator);
  return result;
}
