import {
  AccountLayout,
  createAssociatedTokenAccountInstruction,
  createInitializeMint2Instruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  createSyncNativeInstruction,
  ExtensionType,
  getAssociatedTokenAddressSync,
  getMintLen,
  MINT_SIZE,
  MintLayout,
  NATIVE_MINT,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import BN from "bn.js";
import { BanksClient } from "solana-bankrun";
import { DECIMALS } from "./constants";
const rawAmount = 100_000_000 * 10 ** DECIMALS; // 1 millions

export async function getOrCreateAssociatedTokenAccount(
  banksClient: BanksClient,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey,
  tokenProgram = TOKEN_PROGRAM_ID
) {
  const ataKey = getAssociatedTokenAddressSync(mint, owner, true, tokenProgram);

  const account = await banksClient.getAccount(ataKey);
  if (account === null) {
    const createAtaIx = createAssociatedTokenAccountInstruction(
      payer.publicKey,
      ataKey,
      owner,
      mint,
      tokenProgram
    );
    let transaction = new Transaction();
    const [recentBlockhash] = await banksClient.getLatestBlockhash();
    transaction.recentBlockhash = recentBlockhash;
    transaction.add(createAtaIx);
    transaction.sign(payer);
    await banksClient.processTransaction(transaction);
  }

  return ataKey;
}

export async function createToken(
  banksClient: BanksClient,
  payer: Keypair,
  mintAuthority: PublicKey
): Promise<PublicKey> {
  const mintKeypair = Keypair.generate();
  const rent = await banksClient.getRent();
  const lamports = rent.minimumBalance(BigInt(MINT_SIZE));

  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: mintKeypair.publicKey,
    space: MINT_SIZE,
    lamports: Number(lamports.toString()),
    programId: TOKEN_PROGRAM_ID,
  });

  const initializeMintIx = createInitializeMint2Instruction(
    mintKeypair.publicKey,
    DECIMALS,
    mintAuthority,
    null
  );

  let transaction = new Transaction();
  const [recentBlockhash] = await banksClient.getLatestBlockhash();
  transaction.recentBlockhash = recentBlockhash;
  transaction.add(createAccountIx, initializeMintIx);
  transaction.sign(payer, mintKeypair);

  await banksClient.processTransaction(transaction);

  return mintKeypair.publicKey;
}

export async function wrapSOL(
  banksClient: BanksClient,
  payer: Keypair,
  amount: BN
) {
  const solAta = await getOrCreateAssociatedTokenAccount(
    banksClient,
    payer,
    NATIVE_MINT,
    payer.publicKey
  );

  const solTransferIx = SystemProgram.transfer({
    fromPubkey: payer.publicKey,
    toPubkey: solAta,
    lamports: BigInt(amount.toString()),
  });

  const syncNativeIx = createSyncNativeInstruction(solAta);

  let transaction = new Transaction();
  const [recentBlockhash] = await banksClient.getLatestBlockhash();
  transaction.recentBlockhash = recentBlockhash;
  transaction.add(solTransferIx, syncNativeIx);
  transaction.sign(payer);

  await banksClient.processTransaction(transaction);
}

export async function mintSplTokenTo(
  banksClient: BanksClient,
  payer: Keypair,
  mint: PublicKey,
  mintAuthority: Keypair,
  toWallet: PublicKey
) {
  const destination = await getOrCreateAssociatedTokenAccount(
    banksClient,
    payer,
    mint,
    toWallet
  );

  const mintIx = createMintToInstruction(
    mint,
    destination,
    mintAuthority.publicKey,
    rawAmount
  );

  let transaction = new Transaction();
  const [recentBlockhash] = await banksClient.getLatestBlockhash();
  transaction.recentBlockhash = recentBlockhash;
  transaction.add(mintIx);
  transaction.sign(payer, mintAuthority);

  await banksClient.processTransaction(transaction);
}

export async function getMint(banksClient: BanksClient, mint: PublicKey) {
  const account = await banksClient.getAccount(mint);
  const mintState = MintLayout.decode(account.data);
  return mintState;
}

export async function getTokenAccount(
  banksClient: BanksClient,
  key: PublicKey
) {
  const account = await banksClient.getAccount(key);
  const tokenAccountState = AccountLayout.decode(account.data);
  return tokenAccountState;
}
