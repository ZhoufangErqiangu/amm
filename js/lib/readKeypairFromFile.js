import { Keypair } from "@solana/web3.js";
import fs from "mz/fs.js";

// only at local
export async function readKeypairFromFile(filePath) {
  let keypairString = await fs.readFile(filePath, { encoding: "utf8" });
  let keypairBuffer = Buffer.from(JSON.parse(keypairString));
  return Keypair.fromSecretKey(keypairBuffer);
}
