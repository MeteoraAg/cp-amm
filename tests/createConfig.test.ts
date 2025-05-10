import { BN } from "bn.js";

import { ProgramTestContext } from "solana-bankrun";
import { generateKpAndFund, randomID, startTest } from "./bankrun-utils/common";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  BASIS_POINT_MAX,
  closeConfigIx,
  createConfigIx,
  CreateConfigParams,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  OFFSET,
} from "./bankrun-utils";
import { shlDiv } from "./bankrun-utils/math";

describe("Admin function: Create config", () => {
  let context: ProgramTestContext;
  let admin: Keypair;
  let createConfigParams: CreateConfigParams;

  beforeEach(async () => {
    const root = Keypair.generate();
    context = await startTest(root);
    admin = await generateKpAndFund(context.banksClient, context.payer);

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

  it("Admin create config", async () => {
    await createConfigIx(context.banksClient, admin, createConfigParams);
  });

  it("Admin close config", async () => {
    const config = await createConfigIx(
      context.banksClient,
      admin,
      createConfigParams
    );
    await closeConfigIx(context.banksClient, admin, config);
  });

  it("Admin create config with dynamic fee", async () => {
    // params
    const binStep = new BN(1);
    const binStepU128 = shlDiv(binStep, new BN(BASIS_POINT_MAX), OFFSET);
    const decayPeriod = 5_000;
    const filterPeriod = 2_000;
    const reductionFactor = 5_000;
    const maxVolatilityAccumulator = 350_000;
    const variableFeeControl = 10_000;

    //
    const createConfigParams: CreateConfigParams = {
      index: new BN(Math.floor(Math.random() * 1000)),
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
        dynamicFee: {
          binStep: binStep.toNumber(),
          binStepU128,
          filterPeriod,
          decayPeriod,
          reductionFactor,
          maxVolatilityAccumulator,
          variableFeeControl,
        },
      },
      sqrtMinPrice: new BN(MIN_SQRT_PRICE),
      sqrtMaxPrice: new BN(MAX_SQRT_PRICE),
      vaultConfigKey: PublicKey.default,
      poolCreatorAuthority: PublicKey.default,
      activationType: 0,
      collectFeeMode: 0,
    };

    await createConfigIx(context.banksClient, admin, createConfigParams);
  });
});
