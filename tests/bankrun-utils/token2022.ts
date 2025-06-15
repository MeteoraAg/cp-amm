import {
  createInitializeMint2Instruction,
  createInitializeTransferFeeConfigInstruction,
  ExtensionType,
  getMintLen,
  TOKEN_2022_PROGRAM_ID,
  createInitializeMetadataPointerInstruction,
  createMintToInstruction,
  createInitializePermanentDelegateInstruction,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { BanksClient } from "solana-bankrun";
import { DECIMALS } from "./constants";
import { getOrCreateAssociatedTokenAccount } from "./token";
const rawAmount = 1_000_000 * 10 ** DECIMALS; // 1 millions

interface ExtensionWithInstruction {
  extension: ExtensionType;
  instruction: TransactionInstruction;
}

export function createPermenantDelegateExtensionWithInstruction(
  mint: PublicKey,
  permenantDelegate: PublicKey
): ExtensionWithInstruction {
  return {
    extension: ExtensionType.PermanentDelegate,
    instruction: createInitializePermanentDelegateInstruction(
      mint,
      permenantDelegate,
      TOKEN_2022_PROGRAM_ID
    ),
  };
}

export function createTransferFeeExtensionWithInstruction(
  mint: PublicKey,
  maxFee?: bigint,
  feeBasisPoint?: number,
  transferFeeConfigAuthority?: Keypair,
  withdrawWithheldAuthority?: Keypair
): ExtensionWithInstruction {
  maxFee = maxFee || BigInt(9 * Math.pow(10, DECIMALS));
  feeBasisPoint = feeBasisPoint || 100;
  transferFeeConfigAuthority = transferFeeConfigAuthority || Keypair.generate();
  withdrawWithheldAuthority = withdrawWithheldAuthority || Keypair.generate();
  return {
    extension: ExtensionType.TransferFeeConfig,
    instruction: createInitializeTransferFeeConfigInstruction(
      mint,
      transferFeeConfigAuthority.publicKey,
      withdrawWithheldAuthority.publicKey,
      feeBasisPoint,
      maxFee,
      TOKEN_2022_PROGRAM_ID
    ),
  };
}

export async function createToken2022(
  banksClient: BanksClient,
  payer: Keypair,
  extensions: ExtensionWithInstruction[],
  mintKeypair: Keypair
): Promise<PublicKey> {
  let mintLen = getMintLen(extensions.map((ext) => ext.extension));
  const mintLamports = (await banksClient.getRent()).minimumBalance(
    BigInt(mintLen)
  );
  const transaction = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: mintKeypair.publicKey,
      space: mintLen,
      lamports: Number(mintLamports.toString()),
      programId: TOKEN_2022_PROGRAM_ID,
    }),
    ...extensions.map((ext) => ext.instruction),
    createInitializeMint2Instruction(
      mintKeypair.publicKey,
      DECIMALS,
      payer.publicKey,
      null,
      TOKEN_2022_PROGRAM_ID
    )
  );

  const [recentBlockhash] = await banksClient.getLatestBlockhash();
  transaction.recentBlockhash = recentBlockhash;
  transaction.sign(payer, mintKeypair);

  await banksClient.processTransaction(transaction);

  return mintKeypair.publicKey;
}

export async function mintToToken2022(
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
    toWallet,
    TOKEN_2022_PROGRAM_ID
  );
  const mintIx = createMintToInstruction(
    mint,
    destination,
    mintAuthority.publicKey,
    rawAmount,
    [],
    TOKEN_2022_PROGRAM_ID
  );

  let transaction = new Transaction();
  const [recentBlockhash] = await banksClient.getLatestBlockhash();
  transaction.recentBlockhash = recentBlockhash;
  transaction.add(mintIx);
  transaction.sign(payer, mintAuthority);

  await banksClient.processTransaction(transaction);
}
