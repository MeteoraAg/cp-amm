import { BN } from "bn.js";
import { ProgramTestContext } from "solana-bankrun";
import {
  expectThrowsAsync,
  generateKpAndFund,
  getCpAmmProgramErrorCodeHexString,
  randomID,
  startTest,
} from "./bankrun-utils/common";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  createConfigIx,
  CreateConfigParams,
  createToken,
  deriveConfigAddress,
  getPool,
  initializeCustomizeablePool,
  InitializeCustomizeablePoolParams,
  initializePool,
  InitializePoolParams,
  MAX_SQRT_PRICE,
  MIN_LP_AMOUNT,
  MIN_SQRT_PRICE,
  mintSplTokenTo,
} from "./bankrun-utils";
import { expect } from "chai";

describe("Test create config with max fee", () => {
  let context: ProgramTestContext;
  let admin: Keypair;
  let createConfigParams: CreateConfigParams;
  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;

  beforeEach(async () => {
    const root = Keypair.generate();
    context = await startTest(root);
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
      admin.publicKey
    );

    await mintSplTokenTo(
      context.banksClient,
      context.payer,
      tokenBMint,
      context.payer,
      admin.publicKey
    );

    createConfigParams = {
      index: new BN(randomID()),
      poolFees: {
        baseFee: {
          cliffFeeNumerator: new BN(2_500_000),
          numberOfPeriod: 0,
          reductionFactor: new BN(0),
          periodFrequency: new BN(0),
          feeSchedulerMode: 0,
        },
        maxFeeBps: new BN(5000),
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
  });

  it("Max fee 99%", async () => {
    createConfigParams.poolFees.maxFeeBps = new BN(9900);
    await createConfigIx(context.banksClient, admin, createConfigParams);
  });

  it("Cannot set max fee large than 99%", async () => {
    createConfigParams.poolFees.maxFeeBps = new BN(9901);

    const errorCode = getCpAmmProgramErrorCodeHexString("InvalidFee");
    await expectThrowsAsync(async () => {
      await createConfigIx(context.banksClient, admin, createConfigParams);
    }, errorCode);
  });

  it("Create pool with max fee 99%", async () => {
    createConfigParams.poolFees.maxFeeBps = new BN(9900);
    const cliffFeeNumerator = new BN(990_000_000);
    createConfigParams.poolFees.baseFee.cliffFeeNumerator = cliffFeeNumerator;
    const configAccount = deriveConfigAddress(createConfigParams.index);
    await createConfigIx(context.banksClient, admin, createConfigParams);

    const liquidity = new BN(MIN_LP_AMOUNT);
    const sqrtPrice = new BN(MIN_SQRT_PRICE);

    const initPoolParams: InitializePoolParams = {
      payer: admin,
      creator: admin.publicKey,
      config: configAccount,
      tokenAMint,
      tokenBMint,
      liquidity,
      sqrtPrice,
      activationPoint: null,
    };

    const { pool } = await initializePool(context.banksClient, initPoolParams);

    const poolState = await getPool(context.banksClient, pool);

    expect(poolState.poolFees.baseFee.cliffFeeNumerator.eq(cliffFeeNumerator))
      .to.be.true;
  });

  it("Create customize pool with max fee 99%", async () => {
    const cliffFeeNumerator = new BN(990_000_000);

    const params: InitializeCustomizeablePoolParams = {
      payer: admin,
      creator: admin.publicKey,
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
          cliffFeeNumerator,
          numberOfPeriod: 0,
          reductionFactor: new BN(0),
          periodFrequency: new BN(0),
          feeSchedulerMode: 0,
        },
        maxFeeBps: new BN(9900),
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

    const poolState = await getPool(context.banksClient, pool);

    expect(poolState.poolFees.baseFee.cliffFeeNumerator.eq(cliffFeeNumerator))
      .to.be.true;
  });
});
