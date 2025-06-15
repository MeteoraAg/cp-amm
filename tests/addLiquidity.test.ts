import { AccountLayout } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { expect } from "chai";
import { ProgramTestContext } from "solana-bankrun";
import {
  addLiquidity,
  AddLiquidityParams,
  createConfigIx,
  CreateConfigParams,
  createPosition,
  createToken,
  getPool,
  initializePool,
  InitializePoolParams,
  MAX_SQRT_PRICE,
  MIN_LP_AMOUNT,
  MIN_SQRT_PRICE,
  mintSplTokenTo,
  U64_MAX,
} from "./bankrun-utils";
import { generateKpAndFund, randomID, startTest } from "./bankrun-utils/common";
import {
  createToken2022,
  createTransferFeeExtensionWithInstruction,
  mintToToken2022,
} from "./bankrun-utils/token2022";

describe("Add liquidity", () => {
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
      const createConfigParams: CreateConfigParams = {
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
        new BN(randomID()),
        createConfigParams
      );
    });

    it("Create pool with sqrtPrice equal sqrtMintPrice", async () => {
      const initPoolParams: InitializePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint: tokenAMint,
        tokenBMint: tokenBMint,
        liquidity: MIN_LP_AMOUNT,
        sqrtPrice: MIN_SQRT_PRICE,
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

      const poolState = await getPool(context.banksClient, pool);

      const preTokenAVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenAVault)).data
        ).amount
      );

      const preTokenBVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenBVault)).data
        ).amount
      );

      const addLiquidityParams: AddLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: MIN_LP_AMOUNT,
        tokenAAmountThreshold: U64_MAX,
        tokenBAmountThreshold: U64_MAX,
      };
      await addLiquidity(context.banksClient, addLiquidityParams);

      const postTokenAVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenAVault)).data
        ).amount
      );

      const postTokenBVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenBVault)).data
        ).amount
      );

      expect(preTokenBVaultBalance).eq(postTokenBVaultBalance);

      console.log({ preTokenAVaultBalance, postTokenAVaultBalance });
      console.log({
        preTokenBVaultBalance,
        postTokenBVaultBalance,
      });
    });

    it("Create pool with sqrtPrice equal sqrtMaxPrice", async () => {
      const initPoolParams: InitializePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint: tokenAMint,
        tokenBMint: tokenBMint,
        liquidity: MIN_LP_AMOUNT,
        sqrtPrice: MAX_SQRT_PRICE,
        activationPoint: null,
      };

      const result = await initializePool(context.banksClient, initPoolParams);

      pool = result.pool;
      position = await createPosition(
        context.banksClient,
        creator,
        user.publicKey,
        pool
      );

      const poolState = await getPool(context.banksClient, pool);

      const preTokenAVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenAVault)).data
        ).amount
      );

      const preTokenBVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenBVault)).data
        ).amount
      );

      const addLiquidityParams: AddLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: MIN_LP_AMOUNT,
        tokenAAmountThreshold: U64_MAX,
        tokenBAmountThreshold: U64_MAX,
      };
      await addLiquidity(context.banksClient, addLiquidityParams);

      const postTokenAVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenAVault)).data
        ).amount
      );

      const postTokenBVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenBVault)).data
        ).amount
      );

      console.log({ preTokenAVaultBalance, postTokenAVaultBalance });
      console.log({
        preTokenBVaultBalance,
        postTokenBVaultBalance,
      });

      expect(preTokenAVaultBalance).eq(postTokenAVaultBalance);
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

      const tokenAMintKeypair = Keypair.generate();
      const tokenBMintKeypair = Keypair.generate();

      tokenAMint = tokenAMintKeypair.publicKey;
      tokenBMint = tokenBMintKeypair.publicKey;

      const tokenAExtensions = [
        createTransferFeeExtensionWithInstruction(tokenAMint),
      ];
      const tokenBExtensions = [
        createTransferFeeExtensionWithInstruction(tokenBMint),
      ];
      user = await generateKpAndFund(context.banksClient, context.payer);
      admin = await generateKpAndFund(context.banksClient, context.payer);
      creator = await generateKpAndFund(context.banksClient, context.payer);

      await createToken2022(
        context.banksClient,
        context.payer,
        tokenAExtensions,
        tokenAMintKeypair
      );

      await createToken2022(
        context.banksClient,
        context.payer,
        tokenBExtensions,
        tokenBMintKeypair
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
      const createConfigParams: CreateConfigParams = {
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
        new BN(randomID()),
        createConfigParams
      );
    });

    it("Create pool with sqrtPrice equal sqrtMintPrice", async () => {
      const initPoolParams: InitializePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint: tokenAMint,
        tokenBMint: tokenBMint,
        liquidity: MIN_LP_AMOUNT,
        sqrtPrice: MIN_SQRT_PRICE,
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

      const poolState = await getPool(context.banksClient, pool);

      const preTokenAVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenAVault)).data
        ).amount
      );

      const preTokenBVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenBVault)).data
        ).amount
      );

      const addLiquidityParams: AddLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: MIN_LP_AMOUNT,
        tokenAAmountThreshold: U64_MAX,
        tokenBAmountThreshold: U64_MAX,
      };
      await addLiquidity(context.banksClient, addLiquidityParams);

      const postTokenAVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenAVault)).data
        ).amount
      );

      const postTokenBVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenBVault)).data
        ).amount
      );

      expect(preTokenBVaultBalance).eq(postTokenBVaultBalance);
    });

    it("Create pool with sqrtPrice equal sqrtMaxPrice", async () => {
      const initPoolParams: InitializePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint: tokenAMint,
        tokenBMint: tokenBMint,
        liquidity: MIN_LP_AMOUNT,
        sqrtPrice: MAX_SQRT_PRICE,
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

      const poolState = await getPool(context.banksClient, pool);

      const preTokenAVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenAVault)).data
        ).amount
      );

      const preTokenBVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenBVault)).data
        ).amount
      );

      const addLiquidityParams: AddLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: MIN_LP_AMOUNT,
        tokenAAmountThreshold: U64_MAX,
        tokenBAmountThreshold: U64_MAX,
      };
      await addLiquidity(context.banksClient, addLiquidityParams);

      const postTokenAVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenAVault)).data
        ).amount
      );

      const postTokenBVaultBalance = Number(
        AccountLayout.decode(
          (await context.banksClient.getAccount(poolState.tokenBVault)).data
        ).amount
      );

      console.log({ preTokenAVaultBalance, postTokenAVaultBalance });
      console.log({
        preTokenBVaultBalance,
        postTokenBVaultBalance,
      });

      expect(preTokenAVaultBalance).eq(postTokenAVaultBalance);
    });
  });
});
