import { expect } from "chai";
import { BanksClient, ProgramTestContext } from "solana-bankrun";
import { generateKpAndFund, randomID, startTest } from "./bankrun-utils/common";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  addLiquidity,
  AddLiquidityParams,
  createConfigIx,
  CreateConfigParams,
  createPosition,
  getPool,
  getPosition,
  initializePool,
  InitializePoolParams,
  MIN_LP_AMOUNT,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  swap,
  SwapParams,
  SwapExactOutParams,
  swapExactOut,
  U64_MAX,
  DECIMALS,
  createToken,
  mintSplTokenTo,
} from "./bankrun-utils";
import BN from "bn.js";
import { ExtensionType } from "@solana/spl-token";
import {
  getLiquidityDeltaFromAmountA,
  getLiquidityDeltaFromAmountB,
} from "./bankrun-utils/utils";
import { createToken2022, mintToToken2022 } from "./bankrun-utils/token2022";

describe("Swap Exact Out token", () => {
  describe("SPL Token", () => {
    let context: ProgramTestContext;
    let admin: Keypair;
    let user: Keypair;
    let creator: Keypair;
    let config: PublicKey;
    let liquidity: BN;
    let sqrtPrice: BN;
    let pool: PublicKey;
    let position: PublicKey;
    let inputTokenMint: PublicKey;
    let outputTokenMint: PublicKey;

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);

      user = await generateKpAndFund(context.banksClient, context.payer);
      admin = await generateKpAndFund(context.banksClient, context.payer);
      creator = await generateKpAndFund(context.banksClient, context.payer);

      inputTokenMint = await createToken(
        context.banksClient,
        context.payer,
        context.payer.publicKey
      );
      outputTokenMint = await createToken(
        context.banksClient,
        context.payer,
        context.payer.publicKey
      );

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        inputTokenMint,
        context.payer,
        user.publicKey
      );

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        outputTokenMint,
        context.payer,
        user.publicKey
      );

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        inputTokenMint,
        context.payer,
        creator.publicKey
      );

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        outputTokenMint,
        context.payer,
        creator.publicKey
      );

      // create config
      const createConfigParams: CreateConfigParams = {
        index: new BN(randomID()),
        poolFees: {
          baseFee: {
            cliffFeeNumerator: new BN(2_500_000),
            numberOfPeriod: 0,
            reductionFactor: new BN(0),
            periodFrequency: new BN(0),
            feeSchedulerMode: 0,
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
        collectFeeMode: 0,
      };

      config = await createConfigIx(
        context.banksClient,
        admin,
        createConfigParams
      );

      liquidity = new BN(MIN_LP_AMOUNT);
      sqrtPrice = new BN(MIN_SQRT_PRICE.add(MAX_SQRT_PRICE).divn(2));

      const initPoolParams: InitializePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint: inputTokenMint,
        tokenBMint: outputTokenMint,
        liquidity,
        sqrtPrice,
        activationPoint: null,
      };

      const result = await initializePool(context.banksClient, initPoolParams);
      pool = result.pool;
      position = await createPosition(
        context.banksClient,
        user,
        user.publicKey,
        pool
      );
    });

    it("User swap A->B", async () => {
      const maxAmountA = new BN(10_000_000 * 10 ** DECIMALS);
      const maxAmountB = new BN(10_000_000 * 10 ** DECIMALS);

      const liquidityDeltaFromAmountA = getLiquidityDeltaFromAmountA(
        maxAmountA,
        sqrtPrice,
        MAX_SQRT_PRICE
      );

      const liquidityDeltaFromAmountB = getLiquidityDeltaFromAmountB(
        maxAmountB,
        MIN_SQRT_PRICE,
        sqrtPrice
      );

      console.log({ MIN_SQRT_PRICE, sqrtPrice });

      console.log({ liquidityDeltaFromAmountA, liquidityDeltaFromAmountB });

      const liquidityQ64 = liquidityDeltaFromAmountA.gte(
        liquidityDeltaFromAmountB
      )
        ? liquidityDeltaFromAmountB
        : liquidityDeltaFromAmountA;

      console.log(liquidityQ64);
      const addLiquidityParams: AddLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: liquidityQ64,
        tokenAAmountThreshold: maxAmountA,
        tokenBAmountThreshold: maxAmountB,
      };

      await addLiquidity(context.banksClient, addLiquidityParams);

      // AtoB
      const swapParams: SwapExactOutParams = {
        payer: user,
        pool,
        inputTokenMint,
        outputTokenMint,
        amountOut: new BN(1),
        maximumAmountIn: new BN(U64_MAX),
        referralTokenAccount: null,
      };

      await swapExactOut(context.banksClient, swapParams);

      // BtoA
      const swapParams2: SwapExactOutParams = {
        payer: user,
        pool,
        inputTokenMint: outputTokenMint,
        outputTokenMint: inputTokenMint,
        amountOut: new BN(1),
        maximumAmountIn: new BN(U64_MAX),
        referralTokenAccount: null,
      };

      await swapExactOut(context.banksClient, swapParams2);
    });
  });

  describe("Token 2022", () => {
    let context: ProgramTestContext;
    let admin: Keypair;
    let user: Keypair;
    let creator: Keypair;
    let config: PublicKey;
    let liquidity: BN;
    let sqrtPrice: BN;
    let pool: PublicKey;
    let position: PublicKey;
    let inputTokenMint: PublicKey;
    let outputTokenMint: PublicKey;

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);
      const extensions = [ExtensionType.TransferFeeConfig];
      user = await generateKpAndFund(context.banksClient, context.payer);
      admin = await generateKpAndFund(context.banksClient, context.payer);
      creator = await generateKpAndFund(context.banksClient, context.payer);

      inputTokenMint = await createToken2022(
        context.banksClient,
        context.payer,
        extensions
      );
      outputTokenMint = await createToken2022(
        context.banksClient,
        context.payer,
        extensions
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        inputTokenMint,
        context.payer,
        user.publicKey
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        outputTokenMint,
        context.payer,
        user.publicKey
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        inputTokenMint,
        context.payer,
        creator.publicKey
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        outputTokenMint,
        context.payer,
        creator.publicKey
      );

      // create config
      const createConfigParams: CreateConfigParams = {
        index: new BN(randomID()),
        poolFees: {
          baseFee: {
            cliffFeeNumerator: new BN(2_500_000),
            numberOfPeriod: 0,
            reductionFactor: new BN(0),
            periodFrequency: new BN(0),
            feeSchedulerMode: 0,
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
        collectFeeMode: 0,
      };

      config = await createConfigIx(
        context.banksClient,
        admin,
        createConfigParams
      );

      liquidity = new BN(MIN_LP_AMOUNT);
      sqrtPrice = new BN(MIN_SQRT_PRICE.add(MAX_SQRT_PRICE).divn(2));

      const initPoolParams: InitializePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint: inputTokenMint,
        tokenBMint: outputTokenMint,
        liquidity,
        sqrtPrice,
        activationPoint: null,
      };

      const result = await initializePool(context.banksClient, initPoolParams);
      pool = result.pool;
      position = await createPosition(
        context.banksClient,
        user,
        user.publicKey,
        pool
      );
    });

    it("User swap A->B", async () => {
      const addLiquidityParams: AddLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: new BN(MIN_SQRT_PRICE.muln(2)),
        tokenAAmountThreshold: new BN(200),
        tokenBAmountThreshold: new BN(200),
      };
      await addLiquidity(context.banksClient, addLiquidityParams);

      // AtoB
      const swapParams: SwapExactOutParams = {
        payer: user,
        pool,
        inputTokenMint,
        outputTokenMint,
        amountOut: new BN(1),
        maximumAmountIn: new BN(U64_MAX),
        referralTokenAccount: null,
      };

      await swapExactOut(context.banksClient, swapParams);

      // BtoA
      const swapParams2: SwapExactOutParams = {
        payer: user,
        pool,
        inputTokenMint: outputTokenMint,
        outputTokenMint: inputTokenMint,
        amountOut: new BN(1),
        maximumAmountIn: new BN(U64_MAX),
        referralTokenAccount: null,
      };

      await swapExactOut(context.banksClient, swapParams2);
    });
  });
});
