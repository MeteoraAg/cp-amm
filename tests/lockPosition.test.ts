import { Keypair, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { expect } from "chai";
import { ProgramTestContext } from "solana-bankrun";
import {
  addLiquidity,
  AddLiquidityParams,
  claimPositionFee,
  createConfigIx,
  CreateConfigParams,
  createPosition,
  getPool,
  getPosition,
  getVesting,
  initializePool,
  InitializePoolParams,
  MIN_LP_AMOUNT,
  lockPosition,
  LockPositionParams,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  permanentLockPosition,
  refreshVestings,
  swap,
  SwapParams,
} from "./bankrun-utils";
import {
  setupTestContext,
  startTest,
  warpSlotBy,
} from "./bankrun-utils/common";

describe("Lock position", () => {
  let context: ProgramTestContext;
  let admin: Keypair;
  let user: Keypair;
  let payer: Keypair;
  let config: PublicKey;
  let liquidity: BN;
  let sqrtPrice: BN;
  let pool: PublicKey;
  let position: PublicKey;
  let inputTokenMint: PublicKey;
  let outputTokenMint: PublicKey;
  let liquidityDelta: BN;

  const configId = Math.floor(Math.random() * 1000);
  const vestings: PublicKey[] = [];

  before(async () => {
    context = await startTest();

    const prepareContext = await setupTestContext(
      context.banksClient,
      context.payer
    );
    payer = prepareContext.payer;
    user = prepareContext.user;
    admin = prepareContext.admin;
    inputTokenMint = prepareContext.tokenAMint;
    outputTokenMint = prepareContext.tokenBMint;

    // create config
    const createConfigParams: CreateConfigParams = {
      index: new BN(configId),
      poolFees: {
        tradeFeeNumerator: new BN(10_000_000),
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
    sqrtPrice = new BN(MIN_SQRT_PRICE.muln(2));
    liquidityDelta = new BN(sqrtPrice.mul(new BN(1_000)));

    const initPoolParams: InitializePoolParams = {
      payer: payer,
      creator: prepareContext.poolCreator.publicKey,
      config,
      tokenAMint: prepareContext.tokenAMint,
      tokenBMint: prepareContext.tokenBMint,
      liquidity,
      sqrtPrice,
      activationPoint: null,
    };

    const result = await initializePool(context.banksClient, initPoolParams);
    pool = result.pool;
    position = await createPosition(
      context.banksClient,
      payer,
      user.publicKey,
      pool
    );

    const addLiquidityParams: AddLiquidityParams = {
      owner: user,
      pool,
      position,
      liquidityDelta,
      tokenAAmountThreshold: new BN(2_000_000_000),
      tokenBAmountThreshold: new BN(2_000_000_000),
    };
    await addLiquidity(context.banksClient, addLiquidityParams);
  });

  describe("Lock position", () => {
    const numberOfPeriod = 10;
    const periodFrequency = new BN(1);
    let cliffUnlockLiquidity: BN;
    let liquidityToLock: BN;
    let liquidityPerPeriod: BN;

    it("Partial lock position", async () => {
      const beforePositionState = await getPosition(
        context.banksClient,
        position
      );

      liquidityToLock = beforePositionState.unlockedLiquidity.div(new BN(2));

      cliffUnlockLiquidity = liquidityToLock.div(new BN(2));
      liquidityPerPeriod = liquidityToLock
        .sub(cliffUnlockLiquidity)
        .div(new BN(numberOfPeriod));

      const loss = liquidityToLock.sub(
        cliffUnlockLiquidity.add(liquidityPerPeriod.mul(new BN(numberOfPeriod)))
      );
      cliffUnlockLiquidity = cliffUnlockLiquidity.add(loss);

      const lockPositionParams: LockPositionParams = {
        cliffPoint: null,
        periodFrequency,
        cliffUnlockLiquidity,
        liquidityPerPeriod,
        numberOfPeriod,
        index: vestings.length,
      };

      const vesting = await lockPosition(
        context.banksClient,
        position,
        user,
        user,
        lockPositionParams
      );

      vestings.push(vesting);

      const positionState = await getPosition(context.banksClient, position);
      expect(positionState.vestedLiquidity.eq(liquidityToLock)).to.be.true;

      const vestingState = await getVesting(context.banksClient, vesting);
      expect(!vestingState.cliffPoint.isZero()).to.be.true;
      expect(vestingState.cliffUnlockLiquidity.eq(cliffUnlockLiquidity)).to.be
        .true;
      expect(vestingState.liquidityPerPeriod.eq(liquidityPerPeriod)).to.be.true;
      expect(vestingState.numberOfPeriod).to.be.equal(numberOfPeriod);
      expect(vestingState.position.equals(position)).to.be.true;
      expect(vestingState.totalReleasedLiquidity.isZero()).to.be.true;
      expect(vestingState.periodFrequency.eq(new BN(1))).to.be.true;
    });

    it("Able to claim fee", async () => {
      const swapParams: SwapParams = {
        payer: user,
        pool,
        inputTokenMint,
        outputTokenMint,
        amountIn: new BN(100),
        minimumAmountOut: new BN(0),
        referralTokenAccount: null,
      };

      await swap(context.banksClient, swapParams);

      const claimParams = {
        owner: user,
        pool,
        position,
      };
      await claimPositionFee(context.banksClient, claimParams);
    });

    it("Cliff point", async () => {
      const beforePositionState = await getPosition(
        context.banksClient,
        position
      );

      const beforeVestingState = await getVesting(
        context.banksClient,
        vestings[0]
      );

      await refreshVestings(
        context.banksClient,
        position,
        pool,
        user.publicKey,
        user,
        vestings
      );

      const afterPositionState = await getPosition(
        context.banksClient,
        position
      );

      const afterVestingState = await getVesting(
        context.banksClient,
        vestings[0]
      );

      let vestedLiquidityDelta = beforePositionState.vestedLiquidity.sub(
        afterPositionState.vestedLiquidity
      );

      const positionLiquidityDelta = afterPositionState.unlockedLiquidity.sub(
        beforePositionState.unlockedLiquidity
      );

      expect(positionLiquidityDelta.eq(vestedLiquidityDelta)).to.be.true;

      expect(vestedLiquidityDelta.eq(afterVestingState.cliffUnlockLiquidity)).to
        .be.true;

      vestedLiquidityDelta = afterVestingState.totalReleasedLiquidity.sub(
        beforeVestingState.totalReleasedLiquidity
      );

      expect(vestedLiquidityDelta.eq(afterVestingState.cliffUnlockLiquidity)).to
        .be.true;
    });

    it("Withdraw period", async () => {
      for (let i = 0; i < numberOfPeriod; i++) {
        await warpSlotBy(context, periodFrequency);

        const beforePositionState = await getPosition(
          context.banksClient,
          position
        );

        await refreshVestings(
          context.banksClient,
          position,
          pool,
          user.publicKey,
          user,
          vestings
        );

        const afterPositionState = await getPosition(
          context.banksClient,
          position
        );

        expect(
          afterPositionState.unlockedLiquidity.gt(
            beforePositionState.unlockedLiquidity
          )
        ).to.be.true;
      }

      const vesting = await context.banksClient.getAccount(vestings[0]);
      expect(vesting).is.null;

      const positionState = await getPosition(context.banksClient, position);
      expect(positionState.vestedLiquidity.isZero()).to.be.true;
      expect(positionState.unlockedLiquidity.eq(liquidityDelta)).to.be.true;
    });

    it("Permanent lock position", async () => {
      await permanentLockPosition(context.banksClient, position, user, user);

      const poolState = await getPool(context.banksClient, pool);
      expect(!poolState.permanentLockLiquidity.isZero()).to.be.true;

      const positionState = await getPosition(context.banksClient, position);
      expect(positionState.unlockedLiquidity.isZero()).to.be.true;
      expect(!positionState.permanentLockedLiquidity.isZero()).to.be.true;
    });
  });
});
