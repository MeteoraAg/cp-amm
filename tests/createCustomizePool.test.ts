import { ProgramTestContext } from "solana-bankrun";
import { generateKpAndFund, startTest } from "./bankrun-utils/common";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  InitializeCustomizablePoolParams,
  initializeCustomizablePool,
  MIN_LP_AMOUNT,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  mintSplTokenTo,
  createToken,
  getPool,
} from "./bankrun-utils";
import BN from "bn.js";
import { ExtensionType } from "@solana/spl-token";
import { createToken2022, mintToToken2022 } from "./bankrun-utils/token2022";
import { expect } from "chai";

describe("Initialize customizable pool", () => {
  describe("SPL-Token", () => {
    let context: ProgramTestContext;
    let creator: Keypair;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);
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
        creator.publicKey
      );

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        tokenBMint,
        context.payer,
        creator.publicKey
      );
    });

    it("Initialize customizeable pool with spl token", async () => {
      const params: InitializeCustomizablePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        tokenAMint,
        tokenBMint,
        liquidity: MIN_LP_AMOUNT,
        sqrtPrice: MIN_SQRT_PRICE,
        sqrtMinPrice: MIN_SQRT_PRICE,
        sqrtMaxPrice: MAX_SQRT_PRICE,
        hasAlphaVault: false,
        activationPoint: null,
        poolFees: {
          baseFee: {
            cliffFeeNumerator: new BN(2_500_000),
            firstFactor: 0,
            secondFactor: new BN(0),
            thirdFactor: new BN(0),
            baseFeeMode: 0,
          },
          protocolFeePercent: 20,
          partnerFeePercent: 0,
          referralFeePercent: 20,
          dynamicFee: null,
        },
        activationType: 0,
        collectFeeMode: 0,
      };

      await initializeCustomizablePool(context.banksClient, params);
    });
  });

  describe("Token 2022", () => {
    let context: ProgramTestContext;
    let creator: Keypair;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);
      const extensions = [ExtensionType.TransferFeeConfig];
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
        creator.publicKey
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        tokenBMint,
        context.payer,
        creator.publicKey
      );
    });

    it("Initialize customizeable pool with spl token", async () => {
      const params: InitializeCustomizablePoolParams = {
        payer: creator,
        creator: creator.publicKey,
        tokenAMint,
        tokenBMint,
        liquidity: MIN_LP_AMOUNT,
        sqrtPrice: MIN_SQRT_PRICE,
        sqrtMinPrice: MIN_SQRT_PRICE,
        sqrtMaxPrice: MAX_SQRT_PRICE,
        hasAlphaVault: false,
        activationPoint: null,
        poolFees: {
          baseFee: {
            cliffFeeNumerator: new BN(2_500_000),
            firstFactor: 0,
            secondFactor: new BN(0),
            thirdFactor: new BN(0),
            baseFeeMode: 0,
          },
          protocolFeePercent: 20,
          partnerFeePercent: 0,
          referralFeePercent: 20,
          dynamicFee: null,
        },
        activationType: 0,
        collectFeeMode: 0,
      };

      const { pool } = await initializeCustomizablePool(context.banksClient, params);
      const poolState = await getPool(context.banksClient, pool);
      expect(poolState.version).eq(0);
    });
  });
});
