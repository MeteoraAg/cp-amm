import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { BanksClient, ProgramTestContext, startAnchor } from "solana-bankrun";
import { BASIS_POINT_MAX, CP_AMM_PROGRAM_ID, FEE_DENOMINATOR, MAX_FEE_BPS, MAX_FEE_NUMERATOR, MAX_RATE_LIMITER_DURATION_IN_SECONDS, MAX_RATE_LIMITER_DURATION_IN_SLOTS, MIN_FEE_NUMERATOR } from "./constants";
import BN from "bn.js";
import { BaseFee } from "./cpAmm";

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

export function getRateLimiterParams(
  baseFeeBps: number,
  feeIncrementBps: number,
  referenceAmount: number,
  maxLimiterDuration: number,
  tokenQuoteDecimal: number,
  activationType: number
): BaseFee {
  const cliffFeeNumerator = bpsToFeeNumerator(baseFeeBps)
  const feeIncrementNumerator = bpsToFeeNumerator(feeIncrementBps)

  if (
      baseFeeBps <= 0 ||
      feeIncrementBps <= 0 ||
      referenceAmount <= 0 ||
      maxLimiterDuration <= 0
  ) {
      throw new Error('All rate limiter parameters must be greater than zero')
  }

  if (baseFeeBps > MAX_FEE_BPS) {
      throw new Error(
          `Base fee (${baseFeeBps} bps) exceeds maximum allowed value of ${MAX_FEE_BPS} bps`
      )
  }

  if (feeIncrementBps > MAX_FEE_BPS) {
      throw new Error(
          `Fee increment (${feeIncrementBps} bps) exceeds maximum allowed value of ${MAX_FEE_BPS} bps`
      )
  }

  if (feeIncrementNumerator.gte(new BN(FEE_DENOMINATOR))) {
      throw new Error(
          'Fee increment numerator must be less than FEE_DENOMINATOR'
      )
  }

  const deltaNumerator = new BN(MAX_FEE_NUMERATOR).sub(cliffFeeNumerator)
  const maxIndex = deltaNumerator.div(feeIncrementNumerator)
  if (maxIndex.lt(new BN(1))) {
      throw new Error('Fee increment is too large for the given base fee')
  }

  if (
      cliffFeeNumerator.lt(new BN(MIN_FEE_NUMERATOR)) ||
      cliffFeeNumerator.gt(new BN(MAX_FEE_NUMERATOR))
  ) {
      throw new Error('Base fee must be between 0.01% and 99%')
  }

  const maxDuration =
      activationType == 0
          ? MAX_RATE_LIMITER_DURATION_IN_SLOTS
          : MAX_RATE_LIMITER_DURATION_IN_SECONDS

  if (maxLimiterDuration > maxDuration) {
      throw new Error(
          `Max duration exceeds maximum allowed value of ${maxDuration}`
      )
  }

  const referenceAmountInLamports = new BN(referenceAmount * 10 ** tokenQuoteDecimal)

  return {
      cliffFeeNumerator,
      firstFactor: feeIncrementBps,
      secondFactor: new BN(maxLimiterDuration),
      thirdFactor: new BN(referenceAmountInLamports),
      baseFeeMode: 2, // rate limiter
  }
}

export function calculateRateLimiterFee(params: BaseFee, inputAmount: BN): BN {
  // for input_amount <= reference_amount
  // --> fee = input_amount * cliff_fee_numerator

  // for input_amount > reference_amount

  // let x0 = reference_amount
  // let c = cliff_fee_numerator
  // let i = fee_increment (in basis points)
  // let a = (input_amount - x0) / x0 (integer division)
  // let b = (input_amount - x0) % x0 (remainder)

  // if a < max_index:
  // --> fee = x0 * (c + c*a + i*a*(a+1)/2) + b * (c + i*(a+1))

  // if a ≥ max_index:
  // --> fee = x0 * (c + c*max_index + i*max_index*(max_index+1)/2) + (d*x0 + b) * MAX_FEE
  // where:
  // d = a - max_index
  // MAX_FEE is the maximum allowed fee (9900 bps)

  const { cliffFeeNumerator, thirdFactor, firstFactor } = params

  const feeIncrementNumerator = bpsToFeeNumerator(firstFactor)

  // for input_amount <= reference_amount
  if (inputAmount.lte(thirdFactor)) {
      return inputAmount.mul(cliffFeeNumerator).div(new BN(FEE_DENOMINATOR))
  }

  // for input_amount > reference_amount
  const x0 = thirdFactor
  const c = cliffFeeNumerator
  const i = feeIncrementNumerator

  // calculate a and b
  const diff = inputAmount.sub(x0)
  const a = diff.div(x0)
  const b = diff.mod(x0)

  // calculate max_index
  const maxFeeNumerator = new BN(MAX_FEE_NUMERATOR)
  const deltaNumerator = maxFeeNumerator.sub(cliffFeeNumerator)
  const maxIndex = deltaNumerator.div(feeIncrementNumerator)

  let fee: BN
  if (a.lt(maxIndex)) {
      // if a < max_index
      const numerator1 = c.add(c.mul(a)).add(
          i
              .mul(a)
              .mul(a.add(new BN(1)))
              .div(new BN(2))
      )
      const numerator2 = c.add(i.mul(a.add(new BN(1))))
      const firstFee = x0.mul(numerator1)
      const secondFee = b.mul(numerator2)
      fee = firstFee.add(secondFee)
  } else {
      // if a >= max_index
      const numerator1 = c.add(c.mul(maxIndex)).add(
          i
              .mul(maxIndex)
              .mul(maxIndex.add(new BN(1)))
              .div(new BN(2))
      )
      const numerator2 = maxFeeNumerator
      const firstFee = x0.mul(numerator1)

      const d = a.sub(maxIndex)
      const leftAmount = d.mul(x0).add(b)
      const secondFee = leftAmount.mul(numerator2)
      fee = firstFee.add(secondFee)
  }

  return fee.div(new BN(FEE_DENOMINATOR))
}


export function bpsToFeeNumerator(bps: number): BN {
  return new BN(bps * FEE_DENOMINATOR).divn(BASIS_POINT_MAX)
}

export function feeNumeratorToBps(feeNumerator: BN): number {
  return feeNumerator
      .muln(BASIS_POINT_MAX)
      .div(new BN(FEE_DENOMINATOR))
      .toNumber()
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
   context.warpToSlot(clock.slot + BigInt(slots.toString()));
}
