import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RollupProgram } from "../target/types/rollup_program";
import { PublicKey } from "@solana/web3.js";

describe("rollup-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.RollupProgram as Program<RollupProgram>;

  it.skip("GET", async () => {
    const pda = await program.account.rollupState.fetch(
      new PublicKey("GZVYZfxXowW6zBYQuf5e55e3MZLdwsEdeLifcmoqcfzq")
    );
    console.log(JSON.stringify(pda, null, 2));
  });

  it("Batch 0", async () => {
    const batchNumber = new anchor.BN(0);
    let leaf = Buffer.alloc(32);
    for (let i = 0; i < 32; i++) {
      leaf.writeInt8(1, i);
    }

    const signature = await program.methods
      .submitStateCommitment(batchNumber, leaf)
      .accounts({
        // previousRollupState: new PublicKey(
        //   "GZVYZfxXowW6zBYQuf5e55e3MZLdwsEdeLifcmoqcfzq"
        // ),
        previousRollupState: null,
        validator: program.provider.publicKey,
      })
      .rpc();
    console.log(signature);
  });

  it("Batch 1", async () => {
    const batchNumber = new anchor.BN(1);
    let leaf = Buffer.alloc(32);
    for (let i = 0; i < 32; i++) {
      leaf.writeInt8(2, i);
    }

    const signature = await program.methods
      .submitStateCommitment(batchNumber, leaf)
      .accounts({
        previousRollupState: new PublicKey(
          "GZVYZfxXowW6zBYQuf5e55e3MZLdwsEdeLifcmoqcfzq"
        ),
        // previousRollupState: null,
        validator: program.provider.publicKey,
      })
      .rpc();
    console.log(signature);
  });

  it("Batch 2", async () => {
    const batchNumber = new anchor.BN(2);
    let leaf = Buffer.alloc(32);
    for (let i = 0; i < 32; i++) {
      leaf.writeInt8(3, i);
    }

    const signature = await program.methods
      .submitStateCommitment(batchNumber, leaf)
      .accounts({
        previousRollupState: new PublicKey(
          "14dwJramDKM3wZxhKAukt1Xz9eTPY1iK7vUX8EwHjtEx"
        ),
        // previousRollupState: null,
        validator: program.provider.publicKey,
      })
      .rpc();
    console.log(signature);
  });

  it("Batch 3", async () => {
    const batchNumber = new anchor.BN(3);
    let leaf = Buffer.alloc(32);
    for (let i = 0; i < 32; i++) {
      leaf.writeInt8(4, i);
    }

    const signature = await program.methods
      .submitStateCommitment(batchNumber, leaf)
      .accounts({
        previousRollupState: new PublicKey(
          "CR1BuavmeNAmYP7PYp7piXtTiQ2UUPrPmevkTKbJSM4b"
        ),
        // previousRollupState: null,
        validator: program.provider.publicKey,
      })
      .rpc();
    console.log(signature);
  });

  it("Invalid Dispute", async () => {
    try {
      const batchNumber = new anchor.BN(1);
      let leaf = Buffer.alloc(32);
      for (let i = 0; i < 32; i++) {
        leaf.writeInt8(2, i);
      }
      const seed = batchNumber.toArrayLike(Buffer, "le", 8);
      const rollupState = PublicKey.findProgramAddressSync(
        [seed],
        program.programId
      )[0];
      const tx = await program.methods
        .processFraudProof(leaf, [])
        .accountsPartial({
          validator: program.provider.publicKey,
          rollupState: rollupState,
        })
        .simulate();
      console.log(tx);
    } catch (error) {
      console.error(error);
    }
  });

  it.only("Valid Dispute", async () => {
    try {
      const batchNumber = new anchor.BN(1);
      let leaf = Buffer.alloc(32);
      for (let i = 0; i < 32; i++) {
        leaf.writeInt8(3, i);
      }
      const seed = batchNumber.toArrayLike(Buffer, "le", 8);
      const rollupState = PublicKey.findProgramAddressSync(
        [seed],
        program.programId
      )[0];
      const tx = await program.methods
        .processFraudProof(leaf, [])
        .accountsPartial({
          validator: program.provider.publicKey,
          rollupState: rollupState,
        })
        .simulate();
      console.log(JSON.stringify(tx, null, 2));
    } catch (error) {
      console.error(error);
    }
  });
});
