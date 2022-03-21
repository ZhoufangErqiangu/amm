import {
    Keypair,
    PublicKey,
    Connection,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import BN from 'borsh';
import bs58 from 'bs58';

import { AmmInstruction } from './instruction';
import { readKeypairFromFile } from './lib/readKeypairFromFile';

// mainnet
const rpcUrl = 'https://solana-api.projectserum.com/';
// devnet
// const rpcUrl = 'https://api.devnet.solana.com';
// local
// const rpcUrl = 'http://localhost:8899/';

// comm
const connection = new Connection(rpcUrl, 'finalized');
const idPath = '/home/ubuntu/solana/id.json';

// program
export const ammProgramId = '';
const programId = new PublicKey(ammProgramId);

// only at local
async function getPayer() {
    let keypair = await readKeypairFromFile(idPath);
    console.log('read payer', keypair.publicKey.toBase58());
    return keypair;
}

// debug
async function main() {
    try {
        let args = process.argv.slice(2);
        let index = args[0];
        console.log('js start, index:', index);
        switch (index) {
            case 1: {

            }
            default: {
                console.error('unknow index');
            }
        }
        console.log('js end, index:', index);
    } catch (err) {
        console.error(err);
    }
}

main();
