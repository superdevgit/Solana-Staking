import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SStacking } from "../target/types/s_stacking";

describe("s-stacking", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SStacking as Program<SStacking>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
