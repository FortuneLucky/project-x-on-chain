import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { WenDevProgram } from "../target/types/wen_dev_program";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import * as assert from "assert";
require("dotenv").config();

describe("wen_dev_program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.WenDevProgram as Program<WenDevProgram>;

  it("Can be configured", async () => {
    // Define the payer (who will sign and pay for the transaction).
    const payer = provider.wallet;
    console.log("Program ID from test:", program.programId.toString());

    // Create PDA for the config account using the seed "config".
    const [configPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    // Log PDA details for debugging.
    console.log("Config PDA:", configPda.toString());
    console.log("Bump:", bump);

    // Create a dummy config object to pass as argument.
    const newConfig = {
      authority: payer.publicKey,
      platformBuyFeeBps: 500, // Example fee: 5%
      platformSellFeeBps: 500, // Example fee: 5%
      pegasusBuyFeeBps: 200, // Example fee: 2%
      pegasusSellFeeBps: 200, // Example fee: 2%
      lamportAmountConfig: { range: { min: new anchor.BN(1000), max: new anchor.BN(10000) } },
      tokenSupplyConfig: { range: { min: new anchor.BN(5000), max: new anchor.BN(20000) } },
      tokenDecimalsConfig: { range: { min: 6, max: 9 } },
    };

    // Send the transaction to configure the program.
    const tx = await program.methods
      .configure(newConfig)
      .accounts({
        payer: payer.publicKey,
        // config: configPda, 
        // systemProgram: SystemProgram.programId,
      })
      // .signers([{ ...payer, secretKey: new Uint8Array([38, 156, 17, 116, 178, 4, 201, 125, 228, 216, 219, 62, 60, 142, 111, 82, 6, 40, 227, 27, 87, 53, 106, 46, 93, 8, 75, 165, 13, 59, 242, 56, 6, 184, 75, 229, 50, 54, 247, 157, 97, 68, 156, 41, 116, 81, 172, 132, 46, 22, 243, 30, 127, 50, 5, 136, 187, 198, 232, 35, 167, 57, 230, 54]) }])
      .rpc();

    console.log("Transaction signature:", tx);


    // Fetch the updated config account to validate the changes.
    const configAccount = await program.account.config.fetch(configPda);

    // Assertions to verify configuration
    assert.equal(configAccount.authority.toString(), payer.publicKey.toString());
    assert.equal(configAccount.platformBuyFeeBps, 500);
    assert.equal(configAccount.platformSellFeeBps, 500);
    assert.equal(configAccount.pegasusBuyFeeBps, 200);
    assert.equal(configAccount.pegasusSellFeeBps, 200);
    assert.equal(parseFloat(configAccount.lamportAmountConfig.range.min.toString()), 1000);
    assert.equal(parseFloat(configAccount.lamportAmountConfig.range.max.toString()), 10000);
    assert.equal(parseFloat(configAccount.tokenSupplyConfig.range.min.toString()), 5000);
    assert.equal(parseFloat(configAccount.tokenSupplyConfig.range.max.toString()), 20000);
    assert.equal(parseFloat(configAccount.tokenDecimalsConfig.range.min.toString()), 6);
    assert.equal(parseFloat(configAccount.tokenDecimalsConfig.range.max.toString()), 9);
  });
});


