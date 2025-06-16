import { BN } from "@coral-xyz/anchor";

export enum Rounding {
    Up,
    Down,
  }
  
export function shlDiv(x: BN, y: BN, offset: number) {
    return x.shln(offset).div(y);
}

export function mulDiv(x: BN, y: BN, denominator: BN, rounding: Rounding): BN {
    const { div, mod } = x.mul(y).divmod(denominator);
  
    if (rounding == Rounding.Up && !mod.isZero()) {
      return div.add(new BN(1));
    }
    return div;
  }

