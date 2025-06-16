import { BanksClient, ProgramTestContext } from "solana-bankrun";
import {
  generateKpAndFund,
  startTest,
  warpSlotBy,
} from "./bankrun-utils/common";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  MIN_LP_AMOUNT,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  createToken,
  mintSplTokenTo,
  InitializeCustomizablePoolParams,
  initializeCustomizablePool,
  getPool,
  FEE_DENOMINATOR,
  BaseFee,
} from "./bankrun-utils";
import BN from "bn.js";
import {
  depositAlphaVault,
  fillDammV2,
  getVaultState,
  setupProrataAlphaVault,
} from "./bankrun-utils/alphaVault";
import { NATIVE_MINT } from "@solana/spl-token";
import { mulDiv, Rounding } from "./bankrun-utils/math";
import { expect } from "chai";

describe("Alpha vault with sniper tax", () => {
  describe("Fee Scheduler", () => {
    let context: ProgramTestContext;
    let user: Keypair;
    let creator: Keypair;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);

      user = await generateKpAndFund(context.banksClient, context.payer);
      creator = await generateKpAndFund(context.banksClient, context.payer);

      tokenAMint = await createToken(
        context.banksClient,
        context.payer,
        context.payer.publicKey
      );
      tokenBMint = NATIVE_MINT;

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        tokenAMint,
        context.payer,
        creator.publicKey
      );
    });

    it("Alpha vault can buy before activation point with minimum fee", async () => {
      const baseFee = {
        cliffFeeNumerator: new BN(990_000_000), // 99 %
        firstFactor: 100, // 100 periods
        secondFactor: new BN(1), // 1 slot
        thirdFactor: new BN(9875000),
        baseFeeMode: 0, // fee scheduler Linear mode
      };
      const { pool, alphaVault } = await alphaVaultWithSniperTaxFullflow(
        context,
        user,
        creator,
        tokenAMint,
        tokenBMint,
        baseFee
      );

      const alphaVaultState = await getVaultState(
        context.banksClient,
        alphaVault
      );
      const poolState = await getPool(context.banksClient, pool);
      let totalTradingFee = poolState.metrics.totalLpBFee.add(
        poolState.metrics.totalProtocolBFee
      );
      const totalDeposit = new BN(alphaVaultState.totalDeposit);

      // flat base fee
      // linear fee scheduler
      const feeNumerator = poolState.poolFees.baseFee.cliffFeeNumerator.sub(
        new BN(poolState.poolFees.baseFee.firstFactor).mul(
          poolState.poolFees.baseFee.thirdFactor
        )
      );

      const lpFee = mulDiv(
        totalDeposit,
        feeNumerator,
        new BN(FEE_DENOMINATOR),
        Rounding.Up
      );
      // alpha vault can buy with minimum fee (fee scheduler don't applied)
      // expect total trading fee equal minimum base fee
      expect(totalTradingFee.toNumber()).eq(lpFee.toNumber());
    });
  });

  describe("Rate limiter", () => {
    let context: ProgramTestContext;
    let user: Keypair;
    let creator: Keypair;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);

      user = await generateKpAndFund(context.banksClient, context.payer);
      creator = await generateKpAndFund(context.banksClient, context.payer);

      tokenAMint = await createToken(
        context.banksClient,
        context.payer,
        context.payer.publicKey
      );
      tokenBMint = NATIVE_MINT;

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        tokenAMint,
        context.payer,
        creator.publicKey
      );
    });

    it("Alpha vault can buy before activation point with minimum fee", async () => {
      let referenceAmount = new BN(LAMPORTS_PER_SOL); // 1 SOL
      let maxRateLimiterDuration = new BN(10);
      const baseFee = {
        cliffFeeNumerator: new BN(10_000_000), // 100bps
        firstFactor: 10, // 10 bps
        secondFactor: maxRateLimiterDuration, // 10 slot
        thirdFactor: referenceAmount, // 1 sol
        baseFeeMode: 2, // rate limiter mode
      };
      const { pool, alphaVault } = await alphaVaultWithSniperTaxFullflow(
        context,
        user,
        creator,
        tokenAMint,
        tokenBMint,
        baseFee
      );

      const alphaVaultState = await getVaultState(
        context.banksClient,
        alphaVault
      );
      const poolState = await getPool(context.banksClient, pool);
      let totalTradingFee = poolState.metrics.totalLpBFee.add(
        poolState.metrics.totalProtocolBFee
      );
      const totalDeposit = new BN(alphaVaultState.totalDeposit);
      const feeNumerator = poolState.poolFees.baseFee.cliffFeeNumerator;

      const lpFee = mulDiv(
        totalDeposit,
        feeNumerator,
        new BN(FEE_DENOMINATOR),
        Rounding.Up
      );
      // alpha vault can buy with minimum fee (rate limiter don't applied)
      // expect total trading fee equal minimum base fee
      expect(totalTradingFee.toNumber()).eq(lpFee.toNumber());
    });
  });
});

const alphaVaultWithSniperTaxFullflow = async (
  context: ProgramTestContext,
  user: Keypair,
  creator: Keypair,
  tokenAMint: PublicKey,
  tokenBMint: PublicKey,
  baseFee: BaseFee
): Promise<{ pool: PublicKey; alphaVault: PublicKey }> => {
  let activationPointDiff = 20;
  let startVestingPointDiff = 25;
  let endVestingPointDiff = 30;

  let currentSlot = await context.banksClient.getSlot("processed");
  let activationPoint = new BN(Number(currentSlot) + activationPointDiff);

  console.log("setup permission pool");

  const params: InitializeCustomizablePoolParams = {
    payer: creator,
    creator: creator.publicKey,
    tokenAMint,
    tokenBMint,
    liquidity: MIN_LP_AMOUNT,
    sqrtPrice: MIN_SQRT_PRICE,
    sqrtMinPrice: MIN_SQRT_PRICE,
    sqrtMaxPrice: MAX_SQRT_PRICE,
    hasAlphaVault: true,
    activationPoint,
    poolFees: {
      baseFee,
      protocolFeePercent: 20,
      partnerFeePercent: 0,
      referralFeePercent: 20,
      dynamicFee: null,
    },
    activationType: 0, // slot
    collectFeeMode: 1, // onlyB
  };
  const { pool } = await initializeCustomizablePool(
    context.banksClient,
    params
  );

  console.log("setup prorata vault");
  let startVestingPoint = new BN(Number(currentSlot) + startVestingPointDiff);
  let endVestingPoint = new BN(Number(currentSlot) + endVestingPointDiff);
  let maxBuyingCap = new BN(10 * LAMPORTS_PER_SOL);

  let alphaVault = await setupProrataAlphaVault(context.banksClient, {
    baseMint: tokenAMint,
    quoteMint: tokenBMint,
    pool,
    poolType: 2, // 0: DLMM, 1: Dynamic Pool, 2: DammV2
    startVestingPoint,
    endVestingPoint,
    maxBuyingCap,
    payer: creator,
    escrowFee: new BN(0),
    whitelistMode: 0, // Permissionless
    baseKeypair: creator,
  });

  console.log("User deposit in alpha vault");
  let depositAmount = new BN(10 * LAMPORTS_PER_SOL);
  await depositAlphaVault(context.banksClient, {
    amount: depositAmount,
    ownerKeypair: user,
    alphaVault,
    payer: user,
  });

  // warp slot to pre-activation point
  // alpha vault can buy before activation point
  const preactivationPoint = activationPoint.sub(new BN(5));
  await warpSlotBy(context, preactivationPoint);

  console.log("fill damm v2");
  await fillDammV2(
    context.banksClient,
    pool,
    alphaVault,
    creator,
    maxBuyingCap
  );

  return {
    pool,
    alphaVault,
  };
};
