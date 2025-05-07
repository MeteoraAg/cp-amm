import {
  AnchorProvider,
  BN,
  IdlAccounts,
  IdlTypes,
  Program,
  Wallet,
} from "@coral-xyz/anchor";
import {
  AccountLayout,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getTokenMetadata,
  MintLayout,
  unpackMint,
  ACCOUNT_SIZE,
  ACCOUNT_TYPE_SIZE,
  getExtensionData,
  ExtensionType,
  getMintCloseAuthority,
  MintCloseAuthorityLayout,
  MetadataPointerLayout,
} from "@solana/spl-token";
import { unpack } from "@solana/spl-token-metadata";
import {
  clusterApiUrl,
  ComputeBudgetProgram,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { BanksClient } from "solana-bankrun";
import CpAmmIDL from "../../target/idl/cp_amm.json";
import { CpAmm } from "../../target/types/cp_amm";
import { getOrCreateAssociatedTokenAccount } from "./token";
import {
  deriveClaimFeeOperatorAddress,
  deriveConfigAddress,
  deriveCustomizablePoolAddress,
  derivePoolAddress,
  derivePoolAuthority,
  derivePositionAddress,
  derivePositionNftAccount,
  deriveRewardVaultAddress,
  deriveTokenBadgeAddress,
  deriveTokenVaultAddress,
} from "./accounts";
import { processTransactionMaybeThrow } from "./common";
import { CP_AMM_PROGRAM_ID } from "./constants";
import { assert, expect } from "chai";

export type Pool = IdlAccounts<CpAmm>["pool"];
export type Position = IdlAccounts<CpAmm>["position"];
export type Vesting = IdlAccounts<CpAmm>["vesting"];
export type Config = IdlAccounts<CpAmm>["config"];
export type LockPositionParams = IdlTypes<CpAmm>["vestingParameters"];
export type TokenBadge = IdlAccounts<CpAmm>["tokenBadge"];

export function getSecondKey(key1: PublicKey, key2: PublicKey) {
  const buf1 = key1.toBuffer();
  const buf2 = key2.toBuffer();
  // Buf1 > buf2
  if (Buffer.compare(buf1, buf2) === 1) {
    return buf2;
  }
  return buf1;
}

export function getFirstKey(key1: PublicKey, key2: PublicKey) {
  const buf1 = key1.toBuffer();
  const buf2 = key2.toBuffer();
  // Buf1 > buf2
  if (Buffer.compare(buf1, buf2) === 1) {
    return buf1;
  }
  return buf2;
}

// For create program instruction only
export function createCpAmmProgram() {
  const wallet = new Wallet(Keypair.generate());
  const provider = new AnchorProvider(
    new Connection(clusterApiUrl("devnet")),
    wallet,
    {}
  );
  const program = new Program<CpAmm>(CpAmmIDL as CpAmm, provider);
  return program;
}

export type DynamicFee = {
  binStep: number;
  binStepU128: BN;
  filterPeriod: number;
  decayPeriod: number;
  reductionFactor: number;
  maxVolatilityAccumulator: number;
  variableFeeControl: number;
};

export type BaseFee = {
  cliffFeeNumerator: BN;
  numberOfPeriod: number;
  periodFrequency: BN;
  reductionFactor: BN;
  feeSchedulerMode: number;
};

export type PoolFees = {
  baseFee: BaseFee;
  protocolFeePercent: number;
  partnerFeePercent: number;
  referralFeePercent: number;
  dynamicFee: DynamicFee | null;
};

export type CreateConfigParams = {
  poolFees: PoolFees;
  sqrtMinPrice: BN;
  sqrtMaxPrice: BN;
  vaultConfigKey: PublicKey;
  poolCreatorAuthority: PublicKey;
  activationType: number; // 0: slot, 1: timestamp
  collectFeeMode: number; // 0: BothToken, 1: OnlyTokenB
};

export type CreateDynamicConfigParams = {
  poolCreatorAuthority: PublicKey;
};

export async function createDynamicConfigIx(
  banksClient: BanksClient,
  admin: Keypair,
  index: BN,
  params: CreateDynamicConfigParams
): Promise<PublicKey> {
  const program = createCpAmmProgram();
  const config = deriveConfigAddress(index);
  const transaction = await program.methods
    .createDynamicConfig(index, params)
    .accountsPartial({
      config,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(admin);

  await processTransactionMaybeThrow(banksClient, transaction);

  // Check data
  const configState = await getConfig(banksClient, config);
  expect(configState.poolCreatorAuthority.toString()).eq(
    params.poolCreatorAuthority.toString()
  );

  expect(configState.configType).eq(1); // ConfigType: Dynamic

  return config;
}

export async function createConfigIx(
  banksClient: BanksClient,
  admin: Keypair,
  index: BN,
  params: CreateConfigParams
): Promise<PublicKey> {
  const program = createCpAmmProgram();

  const config = deriveConfigAddress(index);
  const transaction = await program.methods
    .createConfig(index, params)
    .accountsPartial({
      config,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(admin);

  await processTransactionMaybeThrow(banksClient, transaction);

  // Check data
  const configState = await getConfig(banksClient, config);
  expect(configState.vaultConfigKey.toString()).eq(
    params.vaultConfigKey.toString()
  );
  expect(configState.poolCreatorAuthority.toString()).eq(
    params.poolCreatorAuthority.toString()
  );
  expect(configState.activationType).eq(params.activationType);
  expect(configState.collectFeeMode).eq(params.collectFeeMode);
  expect(configState.sqrtMinPrice.toNumber()).eq(
    params.sqrtMinPrice.toNumber()
  );
  expect(configState.sqrtMaxPrice.toString()).eq(
    params.sqrtMaxPrice.toString()
  );
  expect(configState.poolFees.baseFee.cliffFeeNumerator.toNumber()).eq(
    params.poolFees.baseFee.cliffFeeNumerator.toNumber()
  );
  expect(configState.poolFees.baseFee.numberOfPeriod).eq(
    params.poolFees.baseFee.numberOfPeriod
  );
  expect(configState.poolFees.baseFee.reductionFactor.toNumber()).eq(
    params.poolFees.baseFee.reductionFactor.toNumber()
  );
  expect(configState.poolFees.baseFee.feeSchedulerMode).eq(
    params.poolFees.baseFee.feeSchedulerMode
  );
  expect(configState.poolFees.baseFee.periodFrequency.toNumber()).eq(
    params.poolFees.baseFee.periodFrequency.toNumber()
  );
  expect(configState.poolFees.protocolFeePercent).eq(
    params.poolFees.protocolFeePercent
  );
  expect(configState.poolFees.partnerFeePercent).eq(
    params.poolFees.partnerFeePercent
  );
  expect(configState.poolFees.referralFeePercent).eq(
    params.poolFees.referralFeePercent
  );
  expect(configState.configType).eq(0); // ConfigType: Static

  return config;
}

export async function closeConfigIx(
  banksClient: BanksClient,
  admin: Keypair,
  config: PublicKey
) {
  const program = createCpAmmProgram();
  const transaction = await program.methods
    .closeConfig()
    .accountsPartial({
      config,
      admin: admin.publicKey,
      rentReceiver: admin.publicKey,
    })
    .transaction();
  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(admin);

  await processTransactionMaybeThrow(banksClient, transaction);

  const configState = await banksClient.getAccount(config);
  expect(configState).to.be.null;
}

export type CreateTokenBadgeParams = {
  tokenMint: PublicKey;
  admin: Keypair;
};

export async function createTokenBadge(
  banksClient: BanksClient,
  params: CreateTokenBadgeParams
) {
  const { tokenMint, admin } = params;
  const program = createCpAmmProgram();
  const tokenBadge = deriveTokenBadgeAddress(tokenMint);
  const transaction = await program.methods
    .createTokenBadge()
    .accountsPartial({
      tokenBadge,
      tokenMint,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .transaction();
  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(admin);

  await processTransactionMaybeThrow(banksClient, transaction);

  const tokenBadgeState = await getTokenBadge(banksClient, tokenBadge);

  expect(tokenBadgeState.tokenMint.toString()).eq(tokenMint.toString());
}

export type ClaimFeeOperatorParams = {
  admin: Keypair;
  operator: PublicKey;
};
export async function createClaimFeeOperator(
  banksClient: BanksClient,
  params: ClaimFeeOperatorParams
) {
  const program = createCpAmmProgram();
  const { admin, operator } = params;

  const claimFeeOperator = deriveClaimFeeOperatorAddress(operator);
  const transaction = await program.methods
    .createClaimFeeOperator()
    .accountsPartial({
      claimFeeOperator,
      operator,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(admin);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export type CloseFeeOperatorParams = {
  admin: Keypair;
  operator: PublicKey;
  rentReceiver: PublicKey;
};
export async function closeClaimFeeOperator(
  banksClient: BanksClient,
  params: CloseFeeOperatorParams
) {
  const program = createCpAmmProgram();
  const { admin, operator, rentReceiver } = params;

  const claimFeeOperator = deriveClaimFeeOperatorAddress(operator);
  const transaction = await program.methods
    .closeClaimFeeOperator()
    .accountsPartial({
      claimFeeOperator,
      rentReceiver,
      admin: admin.publicKey,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(admin);

  await processTransactionMaybeThrow(banksClient, transaction);

  const account = await banksClient.getAccount(claimFeeOperator);

  expect(account).to.be.null;
}

export type ClaimProtocolFeeParams = {
  operator: Keypair;
  pool: PublicKey;
  treasury: PublicKey;
};
export async function claimProtocolFee(
  banksClient: BanksClient,
  params: ClaimProtocolFeeParams
) {
  const program = createCpAmmProgram();
  const { operator, pool, treasury } = params;
  const poolAuthority = derivePoolAuthority();
  const claimFeeOperator = deriveClaimFeeOperatorAddress(operator.publicKey);
  const poolState = await getPool(banksClient, pool);

  const tokenAProgram = (await banksClient.getAccount(poolState.tokenAMint))
    .owner;
  const tokenBProgram = (await banksClient.getAccount(poolState.tokenBMint))
    .owner;

  const tokenAAccount = await getOrCreateAssociatedTokenAccount(
    banksClient,
    operator,
    poolState.tokenAMint,
    treasury,
    tokenAProgram
  );

  const tokenBAccount = await getOrCreateAssociatedTokenAccount(
    banksClient,
    operator,
    poolState.tokenBMint,
    treasury,
    tokenBProgram
  );

  const transaction = await program.methods
    .claimProtocolFee()
    .accountsPartial({
      poolAuthority,
      pool,
      tokenAVault: poolState.tokenAVault,
      tokenBVault: poolState.tokenBVault,
      tokenAMint: poolState.tokenAMint,
      tokenBMint: poolState.tokenBMint,
      tokenAAccount,
      tokenBAccount,
      claimFeeOperator,
      operator: operator.publicKey,
      tokenAProgram,
      tokenBProgram,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(operator);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export type ClaimPartnerFeeParams = {
  partner: Keypair;
  pool: PublicKey;
  maxAmountA: BN;
  maxAmountB: BN;
};
export async function claimPartnerFee(
  banksClient: BanksClient,
  params: ClaimPartnerFeeParams
) {
  const program = createCpAmmProgram();
  const { partner, pool, maxAmountA, maxAmountB } = params;
  const poolAuthority = derivePoolAuthority();
  const poolState = await getPool(banksClient, pool);
  const tokenAProgram = (await banksClient.getAccount(poolState.tokenAMint))
    .owner;
  const tokenBProgram = (await banksClient.getAccount(poolState.tokenBMint))
    .owner;
  const tokenAAccount = await getOrCreateAssociatedTokenAccount(
    banksClient,
    partner,
    poolState.tokenAMint,
    partner.publicKey,
    tokenAProgram
  );

  const tokenBAccount = await getOrCreateAssociatedTokenAccount(
    banksClient,
    partner,
    poolState.tokenBMint,
    partner.publicKey,
    tokenBProgram
  );
  const transaction = await program.methods
    .claimPartnerFee(maxAmountA, maxAmountB)
    .accountsPartial({
      poolAuthority,
      pool,
      tokenAVault: poolState.tokenAVault,
      tokenBVault: poolState.tokenBVault,
      tokenAMint: poolState.tokenAMint,
      tokenBMint: poolState.tokenBMint,
      tokenAAccount,
      tokenBAccount,
      partner: partner.publicKey,
      tokenAProgram,
      tokenBProgram,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(partner);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export type InitializePoolParams = {
  payer: Keypair;
  creator: PublicKey;
  config: PublicKey;
  tokenAMint: PublicKey;
  tokenBMint: PublicKey;
  liquidity: BN;
  sqrtPrice: BN;
  activationPoint: BN | null;
};

export async function initializePool(
  banksClient: BanksClient,
  params: InitializePoolParams
): Promise<{ pool: PublicKey; position: PublicKey }> {
  const {
    config,
    tokenAMint,
    tokenBMint,
    payer,
    creator,
    liquidity,
    sqrtPrice,
    activationPoint,
  } = params;
  const program = createCpAmmProgram();

  const poolAuthority = derivePoolAuthority();
  const pool = derivePoolAddress(config, tokenAMint, tokenBMint);

  const positionNftKP = Keypair.generate();
  const position = derivePositionAddress(positionNftKP.publicKey);
  const positionNftAccount = derivePositionNftAccount(positionNftKP.publicKey);

  const tokenAVault = deriveTokenVaultAddress(tokenAMint, pool);
  const tokenBVault = deriveTokenVaultAddress(tokenBMint, pool);

  const tokenAProgram = (await banksClient.getAccount(tokenAMint)).owner;
  const tokenBProgram = (await banksClient.getAccount(tokenBMint)).owner;

  const payerTokenA = getAssociatedTokenAddressSync(
    tokenAMint,
    payer.publicKey,
    true,
    tokenAProgram
  );
  const payerTokenB = getAssociatedTokenAddressSync(
    tokenBMint,
    payer.publicKey,
    true,
    tokenBProgram
  );

  let transaction = await program.methods
    .initializePool({
      liquidity: liquidity,
      sqrtPrice: sqrtPrice,
      activationPoint: activationPoint,
    })
    .accountsPartial({
      creator,
      positionNftAccount,
      positionNftMint: positionNftKP.publicKey,
      payer: payer.publicKey,
      config,
      poolAuthority,
      pool,
      position,
      tokenAMint,
      tokenBMint,
      tokenAVault,
      tokenBVault,
      payerTokenA,
      payerTokenB,
      token2022Program: TOKEN_2022_PROGRAM_ID,
      tokenAProgram,
      tokenBProgram,
      systemProgram: SystemProgram.programId,
    })
    .transaction();
  // requires more compute budget than usual
  transaction.add(
    ComputeBudgetProgram.setComputeUnitLimit({
      units: 350_000,
    })
  );
  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(payer, positionNftKP);

  await processTransactionMaybeThrow(banksClient, transaction);

  // validate pool data
  const poolState = await getPool(banksClient, pool);
  expect(poolState.tokenAMint.toString()).eq(tokenAMint.toString());
  expect(poolState.tokenBMint.toString()).eq(tokenBMint.toString());
  expect(poolState.tokenAVault.toString()).eq(tokenAVault.toString());
  expect(poolState.tokenBVault.toString()).eq(tokenBVault.toString());
  expect(poolState.liquidity.toString()).eq(liquidity.toString());
  expect(poolState.sqrtPrice.toString()).eq(sqrtPrice.toString());

  expect(poolState.rewardInfos[0].initialized).eq(0);
  expect(poolState.rewardInfos[1].initialized).eq(0);

  return { pool, position: position };
}

export type InitializePoolWithCustomizeConfigParams = {
  payer: Keypair;
  poolCreatorAuthority: Keypair;
  creator: PublicKey;
  customizeConfigAddress: PublicKey;
  tokenAMint: PublicKey;
  tokenBMint: PublicKey;
  poolFees: PoolFeesParams;
  sqrtMinPrice: BN;
  sqrtMaxPrice: BN;
  hasAlphaVault: boolean;
  liquidity: BN;
  sqrtPrice: BN;
  activationType: number;
  collectFeeMode: number;
  activationPoint: BN | null;
};

export async function initializePoolWithCustomizeConfig(
  banksClient: BanksClient,
  params: InitializePoolWithCustomizeConfigParams
): Promise<{ pool: PublicKey; position: PublicKey }> {
  const {
    tokenAMint,
    tokenBMint,
    payer,
    creator,
    poolCreatorAuthority,
    customizeConfigAddress,
    poolFees,
    hasAlphaVault,
    liquidity,
    sqrtMaxPrice,
    sqrtMinPrice,
    sqrtPrice,
    collectFeeMode,
    activationPoint,
    activationType,
  } = params;
  const program = createCpAmmProgram();

  const poolAuthority = derivePoolAuthority();
  const pool = derivePoolAddress(
    customizeConfigAddress,
    tokenAMint,
    tokenBMint
  );

  const positionNftKP = Keypair.generate();
  const position = derivePositionAddress(positionNftKP.publicKey);
  const positionNftAccount = derivePositionNftAccount(positionNftKP.publicKey);

  const tokenAProgram = (await banksClient.getAccount(tokenAMint)).owner;
  const tokenBProgram = (await banksClient.getAccount(tokenBMint)).owner;

  const tokenAVault = deriveTokenVaultAddress(tokenAMint, pool);
  const tokenBVault = deriveTokenVaultAddress(tokenBMint, pool);

  const payerTokenA = getAssociatedTokenAddressSync(
    tokenAMint,
    payer.publicKey,
    true,
    tokenAProgram
  );
  const payerTokenB = getAssociatedTokenAddressSync(
    tokenBMint,
    payer.publicKey,
    true,
    tokenBProgram
  );

  const transaction = await program.methods
    .initializePoolWithDynamicConfig({
      poolFees,
      sqrtMinPrice,
      sqrtMaxPrice,
      hasAlphaVault,
      liquidity,
      sqrtPrice,
      activationType,
      collectFeeMode,
      activationPoint,
    })
    .accountsPartial({
      creator,
      positionNftAccount,
      positionNftMint: positionNftKP.publicKey,
      payer: payer.publicKey,
      poolCreatorAuthority: poolCreatorAuthority.publicKey,
      config: customizeConfigAddress,
      poolAuthority,
      pool,
      position,
      tokenAMint,
      tokenBMint,
      tokenAVault,
      tokenBVault,
      payerTokenA,
      payerTokenB,
      tokenAProgram,
      tokenBProgram,
      token2022Program: TOKEN_2022_PROGRAM_ID,
    })
    .transaction();
  // requires more compute budget than usual
  transaction.add(
    ComputeBudgetProgram.setComputeUnitLimit({
      units: 350_000,
    })
  );
  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(payer, positionNftKP, poolCreatorAuthority);

  await processTransactionMaybeThrow(banksClient, transaction);

  // validate pool data
  const poolState = await getPool(banksClient, pool);
  expect(poolState.tokenAMint.toString()).eq(tokenAMint.toString());
  expect(poolState.tokenBMint.toString()).eq(tokenBMint.toString());
  expect(poolState.tokenAVault.toString()).eq(tokenAVault.toString());
  expect(poolState.tokenBVault.toString()).eq(tokenBVault.toString());
  expect(poolState.liquidity.toString()).eq(liquidity.toString());
  expect(poolState.sqrtPrice.toString()).eq(sqrtPrice.toString());
  expect(poolState.poolType).eq(1); // Pool type: customize

  expect(poolState.rewardInfos[0].initialized).eq(0);
  expect(poolState.rewardInfos[1].initialized).eq(0);

  return { pool, position: position };
}

export type SetPoolStatusParams = {
  admin: Keypair;
  pool: PublicKey;
  status: number;
};

export async function setPoolStatus(
  banksClient: BanksClient,
  params: SetPoolStatusParams
) {
  const { admin, pool, status } = params;
  const program = createCpAmmProgram();
  const transaction = await program.methods
    .setPoolStatus(status)
    .accountsPartial({
      pool,
      admin: admin.publicKey,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(admin);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export type PoolFeesParams = {
  baseFee: BaseFee;
  protocolFeePercent: number;
  partnerFeePercent: number;
  referralFeePercent: number;
  dynamicFee: DynamicFee | null;
};

export type InitializeCustomizeablePoolParams = {
  payer: Keypair;
  creator: PublicKey;
  tokenAMint: PublicKey;
  tokenBMint: PublicKey;
  poolFees: PoolFeesParams;
  sqrtMinPrice: BN;
  sqrtMaxPrice: BN;
  hasAlphaVault: boolean;
  liquidity: BN;
  sqrtPrice: BN;
  activationType: number;
  collectFeeMode: number;
  activationPoint: BN | null;
};

export async function initializeCustomizeablePool(
  banksClient: BanksClient,
  params: InitializeCustomizeablePoolParams
): Promise<{ pool: PublicKey; position: PublicKey }> {
  const {
    tokenAMint,
    tokenBMint,
    payer,
    creator,
    poolFees,
    hasAlphaVault,
    liquidity,
    sqrtMaxPrice,
    sqrtMinPrice,
    sqrtPrice,
    collectFeeMode,
    activationPoint,
    activationType,
  } = params;
  const program = createCpAmmProgram();

  const poolAuthority = derivePoolAuthority();
  const pool = deriveCustomizablePoolAddress(tokenAMint, tokenBMint);

  const positionNftKP = Keypair.generate();
  const position = derivePositionAddress(positionNftKP.publicKey);
  const positionNftAccount = derivePositionNftAccount(positionNftKP.publicKey);

  const tokenAProgram = (await banksClient.getAccount(tokenAMint)).owner;
  const tokenBProgram = (await banksClient.getAccount(tokenBMint)).owner;

  const tokenAVault = deriveTokenVaultAddress(tokenAMint, pool);
  const tokenBVault = deriveTokenVaultAddress(tokenBMint, pool);

  const payerTokenA = getAssociatedTokenAddressSync(
    tokenAMint,
    payer.publicKey,
    true,
    tokenAProgram
  );
  const payerTokenB = getAssociatedTokenAddressSync(
    tokenBMint,
    payer.publicKey,
    true,
    tokenBProgram
  );

  const transaction = await program.methods
    .initializeCustomizablePool({
      poolFees,
      sqrtMinPrice,
      sqrtMaxPrice,
      hasAlphaVault,
      liquidity,
      sqrtPrice,
      activationType,
      collectFeeMode,
      activationPoint,
    })
    .accountsPartial({
      creator,
      positionNftAccount,
      positionNftMint: positionNftKP.publicKey,
      payer: payer.publicKey,
      poolAuthority,
      pool,
      position,
      tokenAMint,
      tokenBMint,
      tokenAVault,
      tokenBVault,
      payerTokenA,
      payerTokenB,
      token2022Program: TOKEN_2022_PROGRAM_ID,
      tokenAProgram,
      tokenBProgram,
      systemProgram: SystemProgram.programId,
    })
    .transaction();
  // requires more compute budget than usual
  transaction.add(
    ComputeBudgetProgram.setComputeUnitLimit({
      units: 350_000,
    })
  );
  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(payer, positionNftKP);

  await processTransactionMaybeThrow(banksClient, transaction);

  // validate pool data
  const poolState = await getPool(banksClient, pool);
  expect(poolState.tokenAMint.toString()).eq(tokenAMint.toString());
  expect(poolState.tokenBMint.toString()).eq(tokenBMint.toString());
  expect(poolState.tokenAVault.toString()).eq(tokenAVault.toString());
  expect(poolState.tokenBVault.toString()).eq(tokenBVault.toString());
  expect(poolState.liquidity.toString()).eq(liquidity.toString());
  expect(poolState.sqrtPrice.toString()).eq(sqrtPrice.toString());

  expect(poolState.rewardInfos[0].initialized).eq(0);
  expect(poolState.rewardInfos[1].initialized).eq(0);

  return { pool, position: position };
}

export type InitializeRewardParams = {
  payer: Keypair;
  index: number;
  rewardDuration: BN;
  pool: PublicKey;
  rewardMint: PublicKey;
};

export async function initializeReward(
  banksClient: BanksClient,
  params: InitializeRewardParams
): Promise<void> {
  const { index, rewardDuration, pool, rewardMint, payer } = params;
  const program = createCpAmmProgram();

  const poolAuthority = derivePoolAuthority();
  const rewardVault = deriveRewardVaultAddress(pool, index);

  const tokenProgram = (await banksClient.getAccount(rewardMint)).owner;

  const transaction = await program.methods
    .initializeReward(index, rewardDuration, payer.publicKey)
    .accountsPartial({
      pool,
      poolAuthority,
      rewardVault,
      rewardMint,
      admin: payer.publicKey,
      tokenProgram,
      systemProgram: SystemProgram.programId,
    })
    .transaction();
  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(payer);

  await processTransactionMaybeThrow(banksClient, transaction);

  // validate reward data
  const poolState = await getPool(banksClient, pool);
  expect(poolState.rewardInfos[index].initialized).eq(1);
  expect(poolState.rewardInfos[index].vault.toString()).eq(
    rewardVault.toString()
  );
  expect(poolState.rewardInfos[index].mint.toString()).eq(
    rewardMint.toString()
  );
}

export type UpdateRewardDurationParams = {
  index: number;
  admin: Keypair;
  pool: PublicKey;
  newDuration: BN;
};

export async function updateRewardDuration(
  banksClient: BanksClient,
  params: UpdateRewardDurationParams
): Promise<void> {
  const { pool, admin, index, newDuration } = params;
  const program = createCpAmmProgram();
  const transaction = await program.methods
    .updateRewardDuration(index, newDuration)
    .accountsPartial({
      pool,
      admin: admin.publicKey,
    })
    .transaction();
  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(admin);

  await processTransactionMaybeThrow(banksClient, transaction);

  const poolState = await getPool(banksClient, pool);
  expect(poolState.rewardInfos[index].rewardDuration.toNumber()).eq(
    newDuration.toNumber()
  );
}

export type UpdateRewardFunderParams = {
  index: number;
  admin: Keypair;
  pool: PublicKey;
  newFunder: PublicKey;
};

export async function updateRewardFunder(
  banksClient: BanksClient,
  params: UpdateRewardFunderParams
): Promise<void> {
  const { pool, admin, index, newFunder } = params;
  const program = createCpAmmProgram();
  const transaction = await program.methods
    .updateRewardFunder(index, newFunder)
    .accountsPartial({
      pool,
      admin: admin.publicKey,
    })
    .transaction();
  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(admin);

  await processTransactionMaybeThrow(banksClient, transaction);

  const poolState = await getPool(banksClient, pool);
  expect(poolState.rewardInfos[index].funder.toString()).eq(
    newFunder.toString()
  );
}

export type FundRewardParams = {
  funder: Keypair;
  index: number;
  pool: PublicKey;
  carryForward: boolean;
  amount: BN;
};

export async function fundReward(
  banksClient: BanksClient,
  params: FundRewardParams
): Promise<void> {
  const { index, carryForward, pool, funder, amount } = params;
  const program = createCpAmmProgram();

  const poolState = await getPool(banksClient, pool);
  const rewardVault = poolState.rewardInfos[index].vault;
  const tokenProgram = (
    await banksClient.getAccount(poolState.rewardInfos[index].mint)
  ).owner;
  const funderTokenAccount = getAssociatedTokenAddressSync(
    poolState.rewardInfos[index].mint,
    funder.publicKey,
    true,
    tokenProgram
  );

  const rewardVaultPreBalance = Number(
    AccountLayout.decode((await banksClient.getAccount(rewardVault)).data)
      .amount
  );
  const funderPreBalance = Number(
    AccountLayout.decode(
      (await banksClient.getAccount(funderTokenAccount)).data
    ).amount
  );

  const transaction = await program.methods
    .fundReward(index, amount, carryForward)
    .accountsPartial({
      pool,
      rewardVault: poolState.rewardInfos[index].vault,
      rewardMint: poolState.rewardInfos[index].mint,
      funderTokenAccount,
      funder: funder.publicKey,
      tokenProgram,
    })
    .transaction();
  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(funder);

  await processTransactionMaybeThrow(banksClient, transaction);

  const rewardVaultPostBalance = Number(
    AccountLayout.decode((await banksClient.getAccount(rewardVault)).data)
      .amount
  );
  const funderPostBalance = Number(
    AccountLayout.decode(
      (await banksClient.getAccount(funderTokenAccount)).data
    ).amount
  );

  // expect(funderPreBalance - funderPostBalance).eq(amount.toNumber());

  // expect(rewardVaultPostBalance - rewardVaultPreBalance).eq(amount.toNumber());
}

export type ClaimRewardParams = {
  index: number;
  user: Keypair;
  position: PublicKey;
  pool: PublicKey;
};

export async function claimReward(
  banksClient: BanksClient,
  params: ClaimRewardParams
): Promise<void> {
  const { index, pool, user, position } = params;
  const program = createCpAmmProgram();

  const poolState = await getPool(banksClient, pool);
  const positionState = await getPosition(banksClient, position);
  const poolAuthority = derivePoolAuthority();
  const positionNftAccount = derivePositionNftAccount(positionState.nftMint);

  // TODO should use token flag in pool state to get token program ID
  const tokenProgram = (
    await banksClient.getAccount(poolState.rewardInfos[index].mint)
  ).owner;

  const userTokenAccount = await getOrCreateAssociatedTokenAccount(
    banksClient,
    user,
    poolState.rewardInfos[index].mint,
    user.publicKey,
    tokenProgram
  );

  const transaction = await program.methods
    .claimReward(index)
    .accountsPartial({
      pool,
      positionNftAccount,
      rewardVault: poolState.rewardInfos[index].vault,
      rewardMint: poolState.rewardInfos[index].mint,
      poolAuthority,
      position,
      userTokenAccount,
      owner: user.publicKey,
      tokenProgram,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(user);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export type WithdrawIneligibleRewardParams = {
  index: number;
  funder: Keypair;
  pool: PublicKey;
};

export async function withdrawIneligibleReward(
  banksClient: BanksClient,
  params: WithdrawIneligibleRewardParams
): Promise<void> {
  const { index, pool, funder } = params;
  const program = createCpAmmProgram();

  const poolState = await getPool(banksClient, pool);
  const poolAuthority = derivePoolAuthority();
  const tokenProgram = (
    await banksClient.getAccount(poolState.rewardInfos[index].mint)
  ).owner;
  const funderTokenAccount = getAssociatedTokenAddressSync(
    poolState.rewardInfos[index].mint,
    funder.publicKey,
    true,
    tokenProgram
  );

  const transaction = await program.methods
    .withdrawIneligibleReward(index)
    .accountsPartial({
      pool,
      rewardVault: poolState.rewardInfos[index].vault,
      rewardMint: poolState.rewardInfos[index].mint,
      poolAuthority,
      funderTokenAccount,
      funder: funder.publicKey,
      tokenProgram,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(funder);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export async function refreshVestings(
  banksClient: BanksClient,
  position: PublicKey,
  pool: PublicKey,
  owner: PublicKey,
  payer: Keypair,
  vestings: PublicKey[]
) {
  const program = createCpAmmProgram();
  const positionState = await getPosition(banksClient, position);
  const positionNftAccount = derivePositionNftAccount(positionState.nftMint);
  const transaction = await program.methods
    .refreshVesting()
    .accountsPartial({
      position,
      positionNftAccount,
      pool,
      owner,
    })
    .remainingAccounts(
      vestings.map((pubkey) => {
        return {
          isSigner: false,
          isWritable: true,
          pubkey,
        };
      })
    )
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(payer);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export async function permanentLockPosition(
  banksClient: BanksClient,
  position: PublicKey,
  owner: Keypair,
  payer: Keypair
) {
  const program = createCpAmmProgram();

  const positionState = await getPosition(banksClient, position);
  const positionNftAccount = derivePositionNftAccount(positionState.nftMint);

  const transaction = await program.methods
    .permanentLockPosition(positionState.unlockedLiquidity)
    .accountsPartial({
      position,
      positionNftAccount,
      pool: positionState.pool,
      owner: owner.publicKey,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(payer, owner);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export async function lockPosition(
  banksClient: BanksClient,
  position: PublicKey,
  owner: Keypair,
  payer: Keypair,
  params: LockPositionParams
) {
  const program = createCpAmmProgram();
  const positionState = await getPosition(banksClient, position);
  const positionNftAccount = derivePositionNftAccount(positionState.nftMint);

  const vestingKP = Keypair.generate();

  const transaction = await program.methods
    .lockPosition(params)
    .accountsPartial({
      position,
      positionNftAccount,
      vesting: vestingKP.publicKey,
      owner: owner.publicKey,
      pool: positionState.pool,
      program: CP_AMM_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      payer: payer.publicKey,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(payer, owner, vestingKP);

  await processTransactionMaybeThrow(banksClient, transaction);

  return vestingKP.publicKey;
}

export async function createPosition(
  banksClient: BanksClient,
  payer: Keypair,
  owner: PublicKey,
  pool: PublicKey
): Promise<PublicKey> {
  const program = createCpAmmProgram();

  const positionNftKP = Keypair.generate();
  const position = derivePositionAddress(positionNftKP.publicKey);
  const poolAuthority = derivePoolAuthority();
  const positionNftAccount = derivePositionNftAccount(positionNftKP.publicKey);

  const transaction = await program.methods
    .createPosition()
    .accountsPartial({
      owner,
      positionNftMint: positionNftKP.publicKey,
      poolAuthority,
      positionNftAccount,
      payer: payer.publicKey,
      pool,
      position,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(payer, positionNftKP);

  await processTransactionMaybeThrow(banksClient, transaction);

  const positionState = await getPosition(banksClient, position);

  expect(positionState.nftMint.toString()).eq(
    positionNftKP.publicKey.toString()
  );

  const positionNftData = AccountLayout.decode(
    (await banksClient.getAccount(positionNftAccount)).data
  );

  // validate metadata
  const tlvData = (
    await banksClient.getAccount(positionState.nftMint)
  ).data.slice(ACCOUNT_SIZE + ACCOUNT_TYPE_SIZE);
  const metadata = unpack(
    getExtensionData(ExtensionType.TokenMetadata, Buffer.from(tlvData))
  );
  expect(metadata.name).eq("Meteora Position NFT");
  expect(metadata.symbol).eq("MPN");

  // validate metadata pointer
  const metadataAddress = MetadataPointerLayout.decode(
    getExtensionData(ExtensionType.MetadataPointer, Buffer.from(tlvData))
  ).metadataAddress;
  expect(metadataAddress.toString()).eq(positionState.nftMint.toString());

  // validate owner
  expect(positionNftData.owner.toString()).eq(owner.toString());
  expect(Number(positionNftData.amount)).eq(1);
  expect(positionNftData.mint.toString()).eq(
    positionNftKP.publicKey.toString()
  );

  return position;
}

export type AddLiquidityParams = {
  owner: Keypair;
  pool: PublicKey;
  position: PublicKey;
  liquidityDelta: BN;
  tokenAAmountThreshold: BN;
  tokenBAmountThreshold: BN;
};

export async function addLiquidity(
  banksClient: BanksClient,
  params: AddLiquidityParams
) {
  const {
    owner,
    pool,
    position,
    liquidityDelta,
    tokenAAmountThreshold,
    tokenBAmountThreshold,
  } = params;

  const program = createCpAmmProgram();
  const poolState = await getPool(banksClient, pool);
  const positionState = await getPosition(banksClient, position);
  const positionNftAccount = derivePositionNftAccount(positionState.nftMint);

  const tokenAProgram = (await banksClient.getAccount(poolState.tokenAMint))
    .owner;
  const tokenBProgram = (await banksClient.getAccount(poolState.tokenBMint))
    .owner;

  const tokenAAccount = getAssociatedTokenAddressSync(
    poolState.tokenAMint,
    owner.publicKey,
    true,
    tokenAProgram
  );
  const tokenBAccount = getAssociatedTokenAddressSync(
    poolState.tokenBMint,
    owner.publicKey,
    true,
    tokenBProgram
  );
  const tokenAVault = poolState.tokenAVault;
  const tokenBVault = poolState.tokenBVault;
  const tokenAMint = poolState.tokenAMint;
  const tokenBMint = poolState.tokenBMint;

  const transaction = await program.methods
    .addLiquidity({
      liquidityDelta,
      tokenAAmountThreshold,
      tokenBAmountThreshold,
    })
    .accountsPartial({
      pool,
      position,
      positionNftAccount,
      owner: owner.publicKey,
      tokenAAccount,
      tokenBAccount,
      tokenAVault,
      tokenBVault,
      tokenAProgram,
      tokenBProgram,
      tokenAMint,
      tokenBMint,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(owner);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export type RemoveLiquidityParams = AddLiquidityParams;

export async function removeLiquidity(
  banksClient: BanksClient,
  params: RemoveLiquidityParams
) {
  const {
    owner,
    pool,
    position,
    liquidityDelta,
    tokenAAmountThreshold,
    tokenBAmountThreshold,
  } = params;

  const program = createCpAmmProgram();
  const poolState = await getPool(banksClient, pool);
  const positionState = await getPosition(banksClient, position);
  const positionNftAccount = derivePositionNftAccount(positionState.nftMint);

  const poolAuthority = derivePoolAuthority();
  const tokenAProgram = (await banksClient.getAccount(poolState.tokenAMint))
    .owner;
  const tokenBProgram = (await banksClient.getAccount(poolState.tokenBMint))
    .owner;

  const tokenAAccount = getAssociatedTokenAddressSync(
    poolState.tokenAMint,
    owner.publicKey,
    true,
    tokenAProgram
  );
  const tokenBAccount = getAssociatedTokenAddressSync(
    poolState.tokenBMint,
    owner.publicKey,
    true,
    tokenBProgram
  );
  const tokenAVault = poolState.tokenAVault;
  const tokenBVault = poolState.tokenBVault;
  const tokenAMint = poolState.tokenAMint;
  const tokenBMint = poolState.tokenBMint;

  const transaction = await program.methods
    .removeLiquidity({
      liquidityDelta,
      tokenAAmountThreshold,
      tokenBAmountThreshold,
    })
    .accountsPartial({
      poolAuthority,
      pool,
      position,
      positionNftAccount,
      owner: owner.publicKey,
      tokenAAccount,
      tokenBAccount,
      tokenAVault,
      tokenBVault,
      tokenAProgram,
      tokenBProgram,
      tokenAMint,
      tokenBMint,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(owner);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export type RemoveAllLiquidityParams = {
  owner: Keypair;
  pool: PublicKey;
  position: PublicKey;
  tokenAAmountThreshold: BN;
  tokenBAmountThreshold: BN;
};

export async function removeAllLiquidity(
  banksClient: BanksClient,
  params: RemoveAllLiquidityParams
) {
  const {
    owner,
    pool,
    position,
    tokenAAmountThreshold,
    tokenBAmountThreshold,
  } = params;

  const program = createCpAmmProgram();
  const poolState = await getPool(banksClient, pool);
  const positionState = await getPosition(banksClient, position);
  const positionNftAccount = derivePositionNftAccount(positionState.nftMint);

  const poolAuthority = derivePoolAuthority();
  const tokenAProgram = (await banksClient.getAccount(poolState.tokenAMint))
    .owner;
  const tokenBProgram = (await banksClient.getAccount(poolState.tokenBMint))
    .owner;

  const tokenAAccount = getAssociatedTokenAddressSync(
    poolState.tokenAMint,
    owner.publicKey,
    true,
    tokenAProgram
  );
  const tokenBAccount = getAssociatedTokenAddressSync(
    poolState.tokenBMint,
    owner.publicKey,
    true,
    tokenBProgram
  );
  const tokenAVault = poolState.tokenAVault;
  const tokenBVault = poolState.tokenBVault;
  const tokenAMint = poolState.tokenAMint;
  const tokenBMint = poolState.tokenBMint;

  const transaction = await program.methods
    .removeAllLiquidity(tokenAAmountThreshold, tokenBAmountThreshold)
    .accountsPartial({
      poolAuthority,
      pool,
      position,
      positionNftAccount,
      owner: owner.publicKey,
      tokenAAccount,
      tokenBAccount,
      tokenAVault,
      tokenBVault,
      tokenAProgram,
      tokenBProgram,
      tokenAMint,
      tokenBMint,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(owner);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export async function closePosition(
  banksClient: BanksClient,
  params: {
    owner: Keypair;
    pool: PublicKey;
    position: PublicKey;
  }
) {
  const { owner, pool, position } = params;
  const program = createCpAmmProgram();
  const positionState = await getPosition(banksClient, position);
  const poolAuthority = derivePoolAuthority();
  const positionNftAccount = derivePositionNftAccount(positionState.nftMint);

  const transaction = await program.methods
    .closePosition()
    .accountsPartial({
      positionNftMint: positionState.nftMint,
      positionNftAccount,
      pool,
      position,
      poolAuthority,
      rentReceiver: owner.publicKey,
      owner: owner.publicKey,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(owner);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export type SwapParams = {
  payer: Keypair;
  pool: PublicKey;
  inputTokenMint: PublicKey;
  outputTokenMint: PublicKey;
  amountIn: BN;
  minimumAmountOut: BN;
  referralTokenAccount: PublicKey | null;
};

export async function swap(banksClient: BanksClient, params: SwapParams) {
  const {
    payer,
    pool,
    inputTokenMint,
    outputTokenMint,
    amountIn,
    minimumAmountOut,
    referralTokenAccount,
  } = params;

  const program = createCpAmmProgram();
  const poolState = await getPool(banksClient, pool);

  const poolAuthority = derivePoolAuthority();
  const tokenAProgram = (await banksClient.getAccount(poolState.tokenAMint))
    .owner;

  const tokenBProgram = (await banksClient.getAccount(poolState.tokenBMint))
    .owner;
  const inputTokenAccount = getAssociatedTokenAddressSync(
    inputTokenMint,
    payer.publicKey,
    true,
    tokenAProgram
  );
  const outputTokenAccount = getAssociatedTokenAddressSync(
    outputTokenMint,
    payer.publicKey,
    true,
    tokenBProgram
  );
  const tokenAVault = poolState.tokenAVault;
  const tokenBVault = poolState.tokenBVault;
  const tokenAMint = poolState.tokenAMint;
  const tokenBMint = poolState.tokenBMint;

  const transaction = await program.methods
    .swap({
      amountIn,
      minimumAmountOut,
    })
    .accountsPartial({
      poolAuthority,
      pool,
      payer: payer.publicKey,
      inputTokenAccount,
      outputTokenAccount,
      tokenAVault,
      tokenBVault,
      tokenAProgram,
      tokenBProgram,
      tokenAMint,
      tokenBMint,
      referralTokenAccount,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(payer);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export type ClaimpositionFeeParams = {
  owner: Keypair;
  pool: PublicKey;
  position: PublicKey;
};

export async function claimPositionFee(
  banksClient: BanksClient,
  params: ClaimpositionFeeParams
) {
  const { owner, pool, position } = params;

  const program = createCpAmmProgram();
  const poolState = await getPool(banksClient, pool);
  const positionState = await getPosition(banksClient, position);
  const positionNftAccount = derivePositionNftAccount(positionState.nftMint);

  const poolAuthority = derivePoolAuthority();
  const tokenAProgram = (await banksClient.getAccount(poolState.tokenAMint))
    .owner;
  const tokenBProgram = (await banksClient.getAccount(poolState.tokenBMint))
    .owner;

  const tokenAAccount = getAssociatedTokenAddressSync(
    poolState.tokenAMint,
    owner.publicKey,
    true,
    tokenAProgram
  );
  const tokenBAccount = getAssociatedTokenAddressSync(
    poolState.tokenBMint,
    owner.publicKey,
    true,
    tokenBProgram
  );
  const tokenAVault = poolState.tokenAVault;
  const tokenBVault = poolState.tokenBVault;
  const tokenAMint = poolState.tokenAMint;
  const tokenBMint = poolState.tokenBMint;

  const transaction = await program.methods
    .claimPositionFee()
    .accountsPartial({
      poolAuthority,
      owner: owner.publicKey,
      pool,
      position,
      positionNftAccount,
      tokenAAccount,
      tokenBAccount,
      tokenAVault,
      tokenBVault,
      tokenAProgram,
      tokenBProgram,
      tokenAMint,
      tokenBMint,
    })
    .transaction();

  transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
  transaction.sign(owner);

  await processTransactionMaybeThrow(banksClient, transaction);
}

export async function getPool(
  banksClient: BanksClient,
  pool: PublicKey
): Promise<Pool> {
  const program = createCpAmmProgram();
  const account = await banksClient.getAccount(pool);
  return program.coder.accounts.decode("pool", Buffer.from(account.data));
}

export async function getPosition(
  banksClient: BanksClient,
  position: PublicKey
): Promise<Position> {
  const program = createCpAmmProgram();
  const account = await banksClient.getAccount(position);
  return program.coder.accounts.decode("position", Buffer.from(account.data));
}

export async function getVesting(
  banksClient: BanksClient,
  vesting: PublicKey
): Promise<Vesting> {
  const program = createCpAmmProgram();
  const account = await banksClient.getAccount(vesting);
  return program.coder.accounts.decode("vesting", Buffer.from(account.data));
}

export async function getConfig(
  banksClient: BanksClient,
  config: PublicKey
): Promise<Config> {
  const program = createCpAmmProgram();
  const account = await banksClient.getAccount(config);
  return program.coder.accounts.decode("config", Buffer.from(account.data));
}

export function getStakeProgramErrorCodeHexString(errorMessage: String) {
  const error = CpAmmIDL.errors.find(
    (e) =>
      e.name.toLowerCase() === errorMessage.toLowerCase() ||
      e.msg.toLowerCase() === errorMessage.toLowerCase()
  );

  if (!error) {
    throw new Error(`Unknown CP AMM error message / name: ${errorMessage}`);
  }

  return "0x" + error.code.toString(16);
}

export async function getTokenBadge(
  banksClient: BanksClient,
  tokenBadge: PublicKey
): Promise<TokenBadge> {
  const program = createCpAmmProgram();
  const account = await banksClient.getAccount(tokenBadge);
  return program.coder.accounts.decode("tokenBadge", Buffer.from(account.data));
}
