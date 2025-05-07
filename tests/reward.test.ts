import { Clock, ProgramTestContext } from "solana-bankrun";
import { generateKpAndFund, startTest } from "./bankrun-utils/common";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  addLiquidity,
  AddLiquidityParams,
  claimReward,
  createConfigIx,
  CreateConfigParams,
  createPosition,
  fundReward,
  getPool,
  initializePool,
  InitializePoolParams,
  initializeReward,
  InitializeRewardParams,
  MIN_LP_AMOUNT,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  updateRewardDuration,
  updateRewardFunder,
  withdrawIneligibleReward,
  createToken,
  mintSplTokenTo,
} from "./bankrun-utils";
import BN from "bn.js";
import { describe } from "mocha";
import { ExtensionType } from "@solana/spl-token";
import { createToken2022, mintToToken2022 } from "./bankrun-utils/token2022";

describe("Reward unit-testing", () => {
  // SPL-Token
  describe("Reward with SPL-Token", () => {
    let context: ProgramTestContext;
    let creator: Keypair;
    let admin: Keypair;
    let config: PublicKey;
    let funder: Keypair;
    let user: Keypair;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;
    let rewardMint: PublicKey;
    let liquidity: BN;
    let sqrtPrice: BN;
    const configId = Math.floor(Math.random() * 1000);

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);

      user = await generateKpAndFund(context.banksClient, context.payer);
      funder = await generateKpAndFund(context.banksClient, context.payer);
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

      rewardMint = await createToken(
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

      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        rewardMint,
        context.payer,
        funder.publicKey
      );
      await mintSplTokenTo(
        context.banksClient,
        context.payer,
        rewardMint,
        context.payer,
        admin.publicKey
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

    it("Full flow for reward", async () => {
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

      // user create postion and add liquidity
      const position = await createPosition(
        context.banksClient,
        user,
        user.publicKey,
        pool
      );

      const addLiquidityParams: AddLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: new BN(100),
        tokenAAmountThreshold: new BN(200),
        tokenBAmountThreshold: new BN(200),
      };
      await addLiquidity(context.banksClient, addLiquidityParams);

      // init reward
      const index = 0;
      const initRewardParams: InitializeRewardParams = {
        index,
        payer: admin,
        rewardDuration: new BN(24 * 60 * 60),
        pool,
        rewardMint,
      };
      await initializeReward(context.banksClient, initRewardParams);

      // update duration
      await updateRewardDuration(context.banksClient, {
        index,
        admin: admin,
        pool,
        newDuration: new BN(1),
      });

      // update new funder
      await updateRewardFunder(context.banksClient, {
        index,
        admin: admin,
        pool,
        newFunder: funder.publicKey,
      });

      // fund reward
      await fundReward(context.banksClient, {
        index,
        funder: funder,
        pool,
        carryForward: true,
        amount: new BN("100"),
      });

      // claim reward

      await claimReward(context.banksClient, {
        index,
        user,
        pool,
        position,
      });

      // claim ineligible reward
      const poolState = await getPool(context.banksClient, pool);
      // set new timestamp to pass reward duration end
      const timestamp =
        poolState.rewardInfos[index].rewardDurationEnd.addn(5000);
      const currentClock = await context.banksClient.getClock();
      context.setClock(
        new Clock(
          currentClock.slot,
          currentClock.epochStartTimestamp,
          currentClock.epoch,
          currentClock.leaderScheduleEpoch,
          BigInt(timestamp.toString())
        )
      );
      await withdrawIneligibleReward(context.banksClient, {
        index,
        funder,
        pool,
      });
    });
  });

  // SPL-Token2022

  describe("Reward SPL-Token 2022", () => {
    let context: ProgramTestContext;
    let creator: Keypair;
    let config: PublicKey;
    let funder: Keypair;
    let admin: Keypair;
    let user: Keypair;
    let tokenAMint: PublicKey;
    let tokenBMint: PublicKey;
    let rewardMint: PublicKey;
    let liquidity: BN;
    let sqrtPrice: BN;
    const configId = Math.floor(Math.random() * 1000);

    beforeEach(async () => {
      const root = Keypair.generate();
      context = await startTest(root);
      const extensions = [ExtensionType.TransferFeeConfig];

      user = await generateKpAndFund(context.banksClient, context.payer);
      funder = await generateKpAndFund(context.banksClient, context.payer);
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

      rewardMint = await createToken2022(
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

      await mintToToken2022(
        context.banksClient,
        context.payer,
        rewardMint,
        context.payer,
        funder.publicKey
      );

      await mintToToken2022(
        context.banksClient,
        context.payer,
        rewardMint,
        context.payer,
        admin.publicKey
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

    it("Full flow for reward", async () => {
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

      // user create postion and add liquidity
      const position = await createPosition(
        context.banksClient,
        user,
        user.publicKey,
        pool
      );

      const addLiquidityParams: AddLiquidityParams = {
        owner: user,
        pool,
        position,
        liquidityDelta: new BN(100),
        tokenAAmountThreshold: new BN(200),
        tokenBAmountThreshold: new BN(200),
      };
      await addLiquidity(context.banksClient, addLiquidityParams);

      // init reward
      const index = 0;
      const initRewardParams: InitializeRewardParams = {
        index,
        payer: admin,
        rewardDuration: new BN(24 * 60 * 60),
        pool,
        rewardMint,
      };
      await initializeReward(context.banksClient, initRewardParams);

      // update duration
      await updateRewardDuration(context.banksClient, {
        index,
        admin: admin,
        pool,
        newDuration: new BN(1),
      });

      // update new funder
      await updateRewardFunder(context.banksClient, {
        index,
        admin: admin,
        pool,
        newFunder: funder.publicKey,
      });

      console.log("fund reward");
      // fund reward
      await fundReward(context.banksClient, {
        index,
        funder: funder,
        pool,
        carryForward: true,
        amount: new BN("100"),
      });

      // claim reward

      await claimReward(context.banksClient, {
        index,
        user,
        pool,
        position,
      });

      // claim ineligible reward
      const poolState = await getPool(context.banksClient, pool);
      // set new timestamp to pass reward duration end
      const timestamp =
        poolState.rewardInfos[index].rewardDurationEnd.addn(5000);
      const currentClock = await context.banksClient.getClock();
      context.setClock(
        new Clock(
          currentClock.slot,
          currentClock.epochStartTimestamp,
          currentClock.epoch,
          currentClock.leaderScheduleEpoch,
          BigInt(timestamp.toString())
        )
      );
      await withdrawIneligibleReward(context.banksClient, {
        index,
        funder,
        pool,
      });
    });
  });
});
