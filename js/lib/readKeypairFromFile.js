import { Keypair } from "@solana/web3.js";
import fs from 'mz/fs';

// only at local
async function getKeypairFromFile(filePath) {
    let keypairString = await fs.readFile(filePath, { encoding: 'utf8' });
    let keypairBuffer = Buffer.from(JSON.parse(keypairString));
    return new Keypair.fromSecretKey(keypairBuffer);
}
