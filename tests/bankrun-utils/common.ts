import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { BanksClient, ProgramTestContext, startAnchor } from "solana-bankrun";
import { CP_AMM_PROGRAM_ID } from "./constants";
import BN from "bn.js";
import CpAmmIDL from "../../target/idl/cp_amm.json";

export async function startTest(root: Keypair) {
  // Program name need to match fixtures program name
  return startAnchor(
    "./",
    [
      {
        name: "cp_amm",
        programId: new PublicKey(CP_AMM_PROGRAM_ID),
      },
    ],
    [
      {
        address: root.publicKey,
        info: {
          executable: false,
          owner: SystemProgram.programId,
          lamports: LAMPORTS_PER_SOL * 100,
          data: new Uint8Array(),
        },
      },
    ]
  );
}

export async function transferSol(
  banksClient: BanksClient,
  from: Keypair,
  to: PublicKey,
  amount: BN
) {
  const systemTransferIx = SystemProgram.transfer({
    fromPubkey: from.publicKey,
    toPubkey: to,
    lamports: BigInt(amount.toString()),
  });

  let transaction = new Transaction();
  const [recentBlockhash] = await banksClient.getLatestBlockhash();
  transaction.recentBlockhash = recentBlockhash;
  transaction.add(systemTransferIx);
  transaction.sign(from);

  await banksClient.processTransaction(transaction);
}

export async function processTransactionMaybeThrow(
  banksClient: BanksClient,
  transaction: Transaction
) {
  const transactionMeta = await banksClient.tryProcessTransaction(transaction);
  if (transactionMeta.result && transactionMeta.result.length > 0) {
    throw Error(transactionMeta.result);
  }
}

export async function expectThrowsAsync(
  fn: () => Promise<void>,
  errorMessage: String
) {
  try {
    await fn();
  } catch (err) {
    if (!(err instanceof Error)) {
      throw err;
    } else {
      if (!err.message.toLowerCase().includes(errorMessage.toLowerCase())) {
        throw new Error(
          `Unexpected error: ${err.message}. Expected error: ${errorMessage}`
        );
      }
      return;
    }
  }
  throw new Error("Expected an error but didn't get one");
}

export function getCpAmmProgramErrorCodeHexString(errorMessage: String) {
  const error = CpAmmIDL.errors.find(
    (e) =>
      e.name.toLowerCase() === errorMessage.toLowerCase() ||
      e.msg.toLowerCase() === errorMessage.toLowerCase()
  );

  if (!error) {
    throw new Error(`Unknown cp-amm error message / name: ${errorMessage}`);
  }

  return "0x" + error.code.toString(16);
}

export async function generateKpAndFund(
  banksClient: BanksClient,
  rootKeypair: Keypair
): Promise<Keypair> {
  const kp = Keypair.generate();
  await transferSol(
    banksClient,
    rootKeypair,
    kp.publicKey,
    new BN(LAMPORTS_PER_SOL)
  );
  return kp;
}

// async function createAndFundToken2022(
//   banksClient: BanksClient,
//   rootKeypair: Keypair,
//   extensions: ExtensionType[],
//   accounts: PublicKey[]
// ) {
//   const tokenAMintKeypair = Keypair.generate();
//   const tokenBMintKeypair = Keypair.generate();
//   const rewardMintKeypair = Keypair.generate();
//   await createToken2022(
//     banksClient,
//     rootKeypair,
//     tokenAMintKeypair,
//     extensions
//   );
//   await createToken2022(
//     banksClient,
//     rootKeypair,
//     tokenBMintKeypair,
//     extensions
//   );
//   await createToken2022(
//     banksClient,
//     rootKeypair,
//     rewardMintKeypair,
//     extensions
//   );
//   // Mint token A to payer & user
//   for (const account of accounts) {
//     await mintToToken2022(
//       banksClient,
//       rootKeypair,
//       rootKeypair,
//       tokenAMintKeypair.publicKey,
//       account,
//       BigInt(rawAmount)
//     );

//     await mintToToken2022(
//       banksClient,
//       rootKeypair,
//       rootKeypair,
//       tokenBMintKeypair.publicKey,
//       account,
//       BigInt(rawAmount)
//     );

//     await mintToToken2022(
//       banksClient,
//       rootKeypair,
//       rootKeypair,
//       rewardMintKeypair.publicKey,
//       account,
//       BigInt(rawAmount)
//     );

//     await mintToToken2022(
//       banksClient,
//       rootKeypair,
//       rootKeypair,
//       rewardMintKeypair.publicKey,
//       account,
//       BigInt(rawAmount)
//     );
//   }
//   return {
//     tokenAMint: tokenAMintKeypair.publicKey,
//     tokenBMint: tokenBMintKeypair,
//     rewardMint: rewardMintKeypair.publicKey,
//   };
// }

// async function createAndFundSplToken(
//   banksClient: BanksClient,
//   rootKeypair: Keypair,
//   accounts: PublicKey[]
// ) {
//   const tokenAMintKeypair = Keypair.generate();
//   const tokenBMintKeypair = Keypair.generate();
//   const rewardMintKeypair = Keypair.generate();
//   await createToken(
//     banksClient,
//     rootKeypair,
//     tokenAMintKeypair,
//     rootKeypair.publicKey
//   );
//   await createToken(
//     banksClient,
//     rootKeypair,
//     tokenBMintKeypair,
//     rootKeypair.publicKey
//   );
//   await createToken(
//     banksClient,
//     rootKeypair,
//     rewardMintKeypair,
//     rootKeypair.publicKey
//   );
//   // Mint token A to payer & user
//   for (const account of accounts) {
//     mintTo(
//       banksClient,
//       rootKeypair,
//       tokenAMintKeypair.publicKey,
//       rootKeypair,
//       account,
//       BigInt(rawAmount)
//     );

//     mintTo(
//       banksClient,
//       rootKeypair,
//       tokenBMintKeypair.publicKey,
//       rootKeypair,
//       account,
//       BigInt(rawAmount)
//     );

//     await mintTo(
//       banksClient,
//       rootKeypair,
//       rewardMintKeypair.publicKey,
//       rootKeypair,
//       account,
//       BigInt(rawAmount)
//     );

//     await mintTo(
//       banksClient,
//       rootKeypair,
//       rewardMintKeypair.publicKey,
//       rootKeypair,
//       account,
//       BigInt(rawAmount)
//     );
//   }

//   return {
//     tokenAMint: tokenAMintKeypair.publicKey,
//     tokenBMint: tokenBMintKeypair,
//     rewardMint: rewardMintKeypair.publicKey,
//   };
// }

// export async function setupTestContext(
//   banksClient: BanksClient,
//   rootKeypair: Keypair,
//   token2022: boolean,
//   extensions?: ExtensionType[]
// ) {
//   const accounts = await generateKpAndFund(banksClient, rootKeypair, 7);
//   const accountPubkeys = accounts.map((item) => item.publicKey);
//   //
//   let tokens;
//   if (token2022) {
//     tokens = await createAndFundToken2022(
//       banksClient,
//       rootKeypair,
//       extensions,
//       accountPubkeys
//     );
//   } else {
//     tokens = await createAndFundSplToken(
//       banksClient,
//       rootKeypair,
//       accountPubkeys
//     );
//   }

//   return {
//     admin: accounts[0],
//     payer: accounts[1],
//     poolCreator: accounts[2],
//     funder: accounts[3],
//     user: accounts[4],
//     operator: accounts[5],
//     partner: accounts[6],
//     tokenAMint: tokens.tokenAMint,
//     tokenBMint: tokens.tokenBMint,
//     rewardMint: tokens.rewardMint,
//   };
// }

export function randomID(min = 0, max = 10000) {
  return Math.floor(Math.random() * (max - min) + min);
}

export async function warpSlotBy(context: ProgramTestContext, slots: BN) {
  const clock = await context.banksClient.getClock();
  await context.warpToSlot(clock.slot + BigInt(slots.toString()));
}
