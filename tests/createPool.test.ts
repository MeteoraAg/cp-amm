import { expect } from "chai";
import { ProgramTestContext } from "solana-bankrun";
import { generateKpAndFund, startTest } from "./bankrun-utils/common";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  createConfigIx,
  CreateConfigParams,
  getPool,
  initializePool,
  InitializePoolParams,
  MIN_LP_AMOUNT,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  setPoolStatus,
  createToken,
  mintSplTokenTo,
} from "./bankrun-utils";
import BN from "bn.js";
import { ExtensionType } from "@solana/spl-token";
import { createToken2022, mintToToken2022 } from "./bankrun-utils/token2022";

describe("Initialize pool", () => {
  describe("SPL token", () => {
    let context: ProgramTestContext;
    let admin: Keypair;
    let creator: Keypair;
    let config: PublicKey;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;
    let liquidity: BN;
    let sqrtPrice: BN;
    const configId = Math.floor(Math.random() * 1000);

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);
      creator = await generateKpAndFund(context.banksClient, context.payer);
      admin = await generateKpAndFund(context.banksClient, context.payer);

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
        new BN(configId),
        createConfigParams
      );
    });

    it("Initialize pool & update status", async () => {
      liquidity = new BN(MIN_LP_AMOUNT);
      sqrtPrice = new BN(MIN_SQRT_PRICE);

      const initPoolParams: InitializePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint,
        tokenBMint,
        liquidity,
        sqrtPrice,
        activationPoint: null,
      };

      const { pool } = await initializePool(
        context.banksClient,
        initPoolParams
      );

      const newStatus = 1;
      await setPoolStatus(context.banksClient, {
        admin,
        pool,
        status: newStatus,
      });
      const poolState = await getPool(context.banksClient, pool);
      expect(poolState.poolStatus).eq(newStatus);
    });
  });

  describe("Token 2022", () => {
    let context: ProgramTestContext;
    let creator: Keypair;
    let config: PublicKey;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;
    let liquidity: BN;
    let sqrtPrice: BN;
    let admin: Keypair;
    const configId = Math.floor(Math.random() * 1000);

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);
      const extensions = [ExtensionType.TransferFeeConfig];
      creator = await generateKpAndFund(context.banksClient, context.payer);
      admin = await generateKpAndFund(context.banksClient, context.payer);

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
        new BN(configId),
        createConfigParams
      );
    });

    it("Initialize pool", async () => {
      liquidity = new BN(MIN_LP_AMOUNT);
      sqrtPrice = new BN(MIN_SQRT_PRICE);

      const initPoolParams: InitializePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        config,
        tokenAMint,
        tokenBMint,
        liquidity,
        sqrtPrice,
        activationPoint: null,
      };

      const { pool } = await initializePool(
        context.banksClient,
        initPoolParams
      );

      const newStatus = 1;
      await setPoolStatus(context.banksClient, {
        admin,
        pool,
        status: newStatus,
      });
      const poolState = await getPool(context.banksClient, pool);
      expect(poolState.poolStatus).eq(newStatus);
    });
  });
});
