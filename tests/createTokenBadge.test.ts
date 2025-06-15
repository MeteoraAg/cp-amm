import { Keypair, PublicKey } from "@solana/web3.js";
import { ProgramTestContext } from "solana-bankrun";
import { closeTokenBadge, createTokenBadge } from "./bankrun-utils";
import { generateKpAndFund, startTest } from "./bankrun-utils/common";
import {
  createPermenantDelegateExtensionWithInstruction,
  createToken2022,
} from "./bankrun-utils/token2022";
import { ExtensionType } from "@solana/spl-token";

describe("Admin function: Create token badge", () => {
  let context: ProgramTestContext;
  let admin: Keypair;
  let tokenAMint: PublicKey;

  beforeEach(async () => {
    const root = Keypair.generate();
    context = await startTest(root);
    admin = await generateKpAndFund(context.banksClient, context.payer);

    const tokenAMintKeypair = Keypair.generate();
    tokenAMint = tokenAMintKeypair.publicKey;

    const extensions = [
      createPermenantDelegateExtensionWithInstruction(
        tokenAMint,
        admin.publicKey
      ),
    ];

    await createToken2022(
      context.banksClient,
      context.payer,
      extensions,
      tokenAMintKeypair
    );
  });

  it("Admin create token badge", async () => {
    await createTokenBadge(context.banksClient, {
      tokenMint: tokenAMint,
      admin,
    });
  });

  it("Admin close token badge", async () => {
    await createTokenBadge(context.banksClient, {
      tokenMint: tokenAMint,
      admin,
    });
    await closeTokenBadge(context.banksClient, {
      tokenMint: tokenAMint,
      admin,
    });
  });
});
