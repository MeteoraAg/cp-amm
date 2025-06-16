import { ProgramTestContext } from "solana-bankrun";
import {
  expectThrowsAsync,
  generateKpAndFund,
  getCpAmmProgramErrorCodeHexString,
  processTransactionMaybeThrow,
  randomID,
  startTest,
  warpSlotBy,
} from "./bankrun-utils/common";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import {
  InitializeCustomizablePoolParams,
  initializeCustomizablePool,
  MIN_LP_AMOUNT,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  mintSplTokenTo,
  createToken,
  CreateConfigParams,
  createConfigIx,
  InitializePoolParams,
  initializePool,
  getPool,
  swap,
  swapInstruction,
} from "./bankrun-utils";
import BN from "bn.js";
import { assert, expect } from "chai";
import { ANCHOR_ERROR__ACCOUNT_NOT_ENOUGH_KEYS } from "@coral-xyz/anchor-errors";

describe("Rate limiter", () => {
  let context: ProgramTestContext;
  let admin: Keypair;
  let operator: Keypair;
  let partner: Keypair;
  let user: Keypair;
  let poolCreator: Keypair;
  let tokenA: PublicKey;
  let tokenB: PublicKey;

  before(async () => {
    const root = Keypair.generate();
    context = await startTest(root);
    admin = context.payer;
    operator = await generateKpAndFund(context.banksClient, context.payer);
    partner = await generateKpAndFund(context.banksClient, context.payer);
    user = await generateKpAndFund(context.banksClient, context.payer);
    poolCreator = await generateKpAndFund(context.banksClient, context.payer);

    tokenA = await createToken(
      context.banksClient,
      context.payer,
      context.payer.publicKey
    );
    tokenB = await createToken(
      context.banksClient,
      context.payer,
      context.payer.publicKey
    );

    await mintSplTokenTo(
      context.banksClient,
      context.payer,
      tokenA,
      context.payer,
      user.publicKey
    );

    await mintSplTokenTo(
      context.banksClient,
      context.payer,
      tokenB,
      context.payer,
      user.publicKey
    );

    await mintSplTokenTo(
      context.banksClient,
      context.payer,
      tokenA,
      context.payer,
      poolCreator.publicKey
    );

    await mintSplTokenTo(
      context.banksClient,
      context.payer,
      tokenB,
      context.payer,
      poolCreator.publicKey
    );
  });

  it("Rate limiter", async () => {
    let referenceAmount = new BN(LAMPORTS_PER_SOL); // 1 SOL
    let maxRateLimiterDuration = new BN(10);

    const createConfigParams: CreateConfigParams = {
      poolFees: {
        baseFee: {
          cliffFeeNumerator: new BN(10_000_000), // 100bps
          firstFactor: 10, // 10 bps
          secondFactor: maxRateLimiterDuration, // 10 slot
          thirdFactor: referenceAmount, // 1 sol
          baseFeeMode: 2, // rate limiter mode
        },
        protocolFeePercent: 10,
        partnerFeePercent: 0,
        referralFeePercent: 0,
        dynamicFee: null,
      },
      sqrtMinPrice: new BN(MIN_SQRT_PRICE),
      sqrtMaxPrice: new BN(MAX_SQRT_PRICE),
      vaultConfigKey: PublicKey.default,
      poolCreatorAuthority: PublicKey.default,
      activationType: 0,
      collectFeeMode: 1, // onlyB
    };

    let config = await createConfigIx(
      context.banksClient,
      admin,
      new BN(randomID()),
      createConfigParams
    );
    const liquidity = new BN(MIN_LP_AMOUNT);
    const sqrtPrice = new BN(MIN_SQRT_PRICE.muln(2));

    const initPoolParams: InitializePoolParams = {
      payer: poolCreator,
      creator: poolCreator.publicKey,
      config,
      tokenAMint: tokenA,
      tokenBMint: tokenB,
      liquidity,
      sqrtPrice,
      activationPoint: null,
    };
    const { pool } = await initializePool(context.banksClient, initPoolParams);
    let poolState = await getPool(context.banksClient, pool);

    // swap with 1 SOL

    await swap(context.banksClient, {
      payer: poolCreator,
      pool,
      inputTokenMint: tokenB,
      outputTokenMint: tokenA,
      amountIn: referenceAmount,
      minimumAmountOut: new BN(0),
      referralTokenAccount: null,
    });

    poolState = await getPool(context.banksClient, pool);

    let totalTradingFee = poolState.metrics.totalLpBFee.add(
      poolState.metrics.totalProtocolBFee
    );

    expect(totalTradingFee.toNumber()).eq(
      referenceAmount.div(new BN(100)).toNumber()
    );

    // swap with 2 SOL

    await swap(context.banksClient, {
      payer: poolCreator,
      pool,
      inputTokenMint: tokenB,
      outputTokenMint: tokenA,
      amountIn: referenceAmount.mul(new BN(2)),
      minimumAmountOut: new BN(0),
      referralTokenAccount: null,
    });

    poolState = await getPool(context.banksClient, pool);

    let totalTradingFee1 = poolState.metrics.totalLpBFee.add(
      poolState.metrics.totalProtocolBFee
    );
    let deltaTradingFee = totalTradingFee1.sub(totalTradingFee);

    expect(deltaTradingFee.toNumber()).gt(
      referenceAmount.mul(new BN(2)).div(new BN(100)).toNumber()
    );

    // wait until time pass the 10 slot
    await warpSlotBy(context, maxRateLimiterDuration.add(new BN(1)));

    // swap with 2 SOL

    await swap(context.banksClient, {
      payer: poolCreator,
      pool,
      inputTokenMint: tokenB,
      outputTokenMint: tokenA,
      amountIn: referenceAmount.mul(new BN(2)),
      minimumAmountOut: new BN(0),
      referralTokenAccount: null,
    });

    poolState = await getPool(context.banksClient, pool);

    let totalTradingFee2 = poolState.metrics.totalLpBFee.add(
      poolState.metrics.totalProtocolBFee
    );
    let deltaTradingFee1 = totalTradingFee2.sub(totalTradingFee1);
    expect(deltaTradingFee1.toNumber()).eq(
      referenceAmount.mul(new BN(2)).div(new BN(100)).toNumber()
    );
  });

  it("Try to send multiple instructions", async () => {
    let referenceAmount = new BN(LAMPORTS_PER_SOL); // 1 SOL
    let maxRateLimiterDuration = new BN(10);
    const liquidity = new BN(MIN_LP_AMOUNT);
    const sqrtPrice = new BN(MIN_SQRT_PRICE.muln(2));

    const initPoolParams: InitializeCustomizablePoolParams = {
      payer: poolCreator,
      creator: poolCreator.publicKey,
      tokenAMint: tokenA,
      tokenBMint: tokenB,
      poolFees: {
        baseFee: {
          cliffFeeNumerator: new BN(10_000_000), // 100bps
          firstFactor: 10, // 10 bps
          secondFactor: maxRateLimiterDuration, // 10 slot
          thirdFactor: referenceAmount, // 1 sol
          baseFeeMode: 2, // rate limiter mode
        },
        protocolFeePercent: 20,
        partnerFeePercent: 0,
        referralFeePercent: 20,
        dynamicFee: null,
      },
      sqrtMinPrice: new BN(MIN_SQRT_PRICE),
      sqrtMaxPrice: new BN(MAX_SQRT_PRICE),
      liquidity,
      sqrtPrice,
      hasAlphaVault: false,
      activationType: 0,
      collectFeeMode: 1, // onlyB
      activationPoint: null,
    };
    const { pool } = await initializeCustomizablePool(
      context.banksClient,
      initPoolParams
    );

    // swap with 1 SOL
    const swapIx = await swapInstruction(context.banksClient, {
      payer: poolCreator,
      pool,
      inputTokenMint: tokenB,
      outputTokenMint: tokenA,
      amountIn: referenceAmount,
      minimumAmountOut: new BN(0),
      referralTokenAccount: null,
    });

    let transaction = new Transaction();
    for (let i = 0; i < 2; i++) {
      transaction.add(swapIx);
    }

    transaction.recentBlockhash = (
      await context.banksClient.getLatestBlockhash()
    )[0];
    transaction.sign(poolCreator);

    const errorCode = getCpAmmProgramErrorCodeHexString("FailToValidateSingleSwapInstruction")
    await expectThrowsAsync(async ()=>{
      await processTransactionMaybeThrow(context.banksClient, transaction); 
    }, errorCode)
  });
});
