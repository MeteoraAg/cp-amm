import { expect } from "chai";
import { BanksClient, ProgramTestContext } from "solana-bankrun";
import {
  LOCAL_ADMIN_KEYPAIR,
  createUsersAndFund,
  setupTestContext,
  startTest,
  transferSol,
} from "./bankrun-utils/common";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { createMint, wrapSOL } from "./bankrun-utils/token";
import {
  addLiquidity,
  AddLiquidityParams,
  claimPositionFee,
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
} from "./bankrun-utils";
import BN from "bn.js";

describe("Claim position fee", () => {
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
  const configId = Math.floor(Math.random() * 1000);

  beforeEach(async () => {
    context = await startTest();

    const prepareContext = await setupTestContext(
      context.banksClient,
      context.payer,
      false
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
        baseFee: {
          cliffFeeNumerator: new BN(2_500_000),
          numberOfPeriod: 0,
          deltaPerPeriod: new BN(0),
          periodFrequency: new BN(0),
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
    sqrtPrice = new BN(MIN_SQRT_PRICE.muln(2));

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
  });

  it("User claim position fee", async () => {
    const addLiquidityParams: AddLiquidityParams = {
      owner: user,
      pool,
      position,
      liquidityDelta: new BN(MIN_SQRT_PRICE.muln(30)),
      tokenAAmountThreshold: new BN(200),
      tokenBAmountThreshold: new BN(200),
    };
    await addLiquidity(context.banksClient, addLiquidityParams);

    const swapParams: SwapParams = {
      payer: user,
      pool,
      inputTokenMint,
      outputTokenMint,
      amountIn: new BN(10),
      minimumAmountOut: new BN(0),
      referralTokenAccount: null,
    };

    await swap(context.banksClient, swapParams);

    // claim position fee
    const claimParams = {
      owner: user,
      pool,
      position,
    };
    await claimPositionFee(context.banksClient, claimParams);
  });
});
