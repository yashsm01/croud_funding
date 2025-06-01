import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Croudfunding } from "../target/types/croudfunding";

describe("croudfunding", () => {
  // Configure the client to use the local cluster.

  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.croudfunding as Program<Croudfunding>;

});
