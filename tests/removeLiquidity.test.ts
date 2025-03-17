import { ProgramTestContext } from "solana-bankrun";
import { generateKpAndFund, randomID, startTest } from "./bankrun-utils/common";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  addLiquidity,
  createConfigIx,
  createPosition,
  initializePool,
  MIN_LP_AMOUNT,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  removeLiquidity,
  U64_MAX,
  mintSplTokenTo,
  createToken,
} from "./bankrun-utils";
import BN from "bn.js";
import { ExtensionType } from "@solana/spl-token";
import { createToken2022, mintToToken2022 } from "./bankrun-utils/token2022";

describe("Remove liquidity", () => {
  describe("SPL Token", () => {
    let context: ProgramTestContext;
    let admin: Keypair;
    let user: Keypair;
    let creator: Keypair;
    let config: PublicKey;
    let pool: PublicKey;
    let position: PublicKey;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);

      user = await generateKpAndFund(context.banksClient, context.payer);
      admin = await generateKpAndFund(context.banksClient, context.payer);
      creator = await generateKpAndFund(context.banksClient, context.payer);

      tokenAMint = await createToken(
        context.banksClient,
        context.payer,
        context.payer.publicKey
      );
      tokenBMint = await createToken(
        context.banksClient,
        context.payer,
        context.payer.publicKey
      );

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        tokenAMint,
        context.payer,
        user.publicKey
      );

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        tokenBMint,
        context.payer,
        user.publicKey
      );

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        tokenAMint,
        context.payer,
        creator.publicKey
      );

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        tokenBMint,
        context.payer,
        creator.publicKey
      );

      // create config
      const createConfigParams = {
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

      const initPoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint,
        tokenBMint,
        liquidity: new BN(MIN_LP_AMOUNT),
        sqrtPrice: new BN(MIN_SQRT_PRICE),
        activationPoint: null,
      };

      const result = await initializePool(context.banksClient, initPoolParams);
      pool = result.pool;
    });

    it("User remove liquidity", async () => {
      // create a position
      const position = await createPosition(
        context.banksClient,
        user,
        user.publicKey,
        pool
      );

      // add liquidity
      let liquidity = new BN("100000000000");
      const addLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: liquidity,
        tokenAAmountThreshold: U64_MAX,
        tokenBAmountThreshold: U64_MAX,
      };
      await addLiquidity(context.banksClient, addLiquidityParams);
      // return

      // remove liquidity

      const removeLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: liquidity,
        tokenAAmountThreshold: new BN(0),
        tokenBAmountThreshold: new BN(0),
      };
      await removeLiquidity(context.banksClient, removeLiquidityParams);
    });
  });

  describe("Token 2022", () => {
    let context: ProgramTestContext;
    let admin: Keypair;
    let user: Keypair;
    let config: PublicKey;
    let pool: PublicKey;
    let position: PublicKey;
    let creator: Keypair;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);
      const extensions = [ExtensionType.TransferFeeConfig];
      user = await generateKpAndFund(context.banksClient, context.payer);
      admin = await generateKpAndFund(context.banksClient, context.payer);
      creator = await generateKpAndFund(context.banksClient, context.payer);

      tokenAMint = await createToken2022(
        context.banksClient,
        context.payer,
        extensions
      );
      tokenBMint = await createToken2022(
        context.banksClient,
        context.payer,
        extensions
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        tokenAMint,
        context.payer,
        user.publicKey
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        tokenBMint,
        context.payer,
        user.publicKey
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        tokenAMint,
        context.payer,
        creator.publicKey
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        tokenBMint,
        context.payer,
        creator.publicKey
      );

      // create config
      const createConfigParams = {
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

      const initPoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint: tokenAMint,
        tokenBMint: tokenBMint,
        liquidity: new BN(MIN_LP_AMOUNT),
        sqrtPrice: new BN(MIN_SQRT_PRICE),
        activationPoint: null,
      };

      const result = await initializePool(context.banksClient, initPoolParams);
      pool = result.pool;
    });

    it("User remove liquidity", async () => {
      // create a position
      const position = await createPosition(
        context.banksClient,
        user,
        user.publicKey,
        pool
      );

      // add liquidity
      let liquidity = new BN("100000000000");
      const addLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: liquidity,
        tokenAAmountThreshold: U64_MAX,
        tokenBAmountThreshold: U64_MAX,
      };
      await addLiquidity(context.banksClient, addLiquidityParams);
      // return

      const removeLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: liquidity,
        tokenAAmountThreshold: new BN(0),
        tokenBAmountThreshold: new BN(0),
      };
      await removeLiquidity(context.banksClient, removeLiquidityParams);
    });
  });
});
