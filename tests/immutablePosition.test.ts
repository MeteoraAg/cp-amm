import { ProgramTestContext } from "solana-bankrun";
import { generateKpAndFund, startTest } from "./bankrun-utils/common";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  InitializeCustomizeablePoolParams,
  initializeCustomizeablePool,
  MIN_LP_AMOUNT,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  createTokenBadge,
  createPosition,
  CreateConfigParams,
  createConfigIx,
  InitializePoolParams,
  initializePool,
} from "./bankrun-utils";
import BN from "bn.js";
import { ExtensionType } from "@solana/spl-token";
import { createToken2022, mintToToken2022 } from "./bankrun-utils/token2022";

describe("Immutable position owner", () => {
  let context: ProgramTestContext;
  let creator: Keypair;
  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;

  beforeEach(async () => {
    const root = Keypair.generate();
    context = await startTest(root);
    const extensions = [ExtensionType.DefaultAccountState];
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

    await createTokenBadge(context.banksClient, {
      tokenMint: tokenAMint,
      admin: context.payer,
      immutablePosition: 1, // immutable
    });

    await createTokenBadge(context.banksClient, {
      tokenMint: tokenBMint,
      admin: context.payer,
      immutablePosition: 0, // mutable
    });
  });

  it("create customize pool with immutable position owner", async () => {
    const params: InitializeCustomizeablePoolParams = {
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
          numberOfPeriod: 0,
          reductionFactor: new BN(0),
          periodFrequency: new BN(0),
          feeSchedulerMode: 0,
        },
        protocolFeePercent: 20,
        partnerFeePercent: 0,
        referralFeePercent: 20,
        dynamicFee: null,
      },
      activationType: 0,
      collectFeeMode: 0,
    };

    const { pool } = await initializeCustomizeablePool(
      context.banksClient,
      params
    );

    await createPosition(
      context.banksClient,
      context.payer,
      context.payer.publicKey,
      pool
    );
  });
  it("create pool with immutable position owner", async () => {
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

    const config = await createConfigIx(
      context.banksClient,
      context.payer,
      new BN(Math.floor(Math.random() * 1000)),
      createConfigParams
    );

    const initPoolParams: InitializePoolParams = {
      payer: creator,
      creator: creator.publicKey,
      config,
      tokenAMint,
      tokenBMint,
      liquidity: new BN(MIN_LP_AMOUNT),
      sqrtPrice: new BN(MIN_SQRT_PRICE),
      activationPoint: null,
    };

    const { pool } = await initializePool(context.banksClient, initPoolParams);
    await createPosition(
      context.banksClient,
      context.payer,
      context.payer.publicKey,
      pool
    );
  });
});
