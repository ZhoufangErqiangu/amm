import {
    Keypair,
    PublicKey,
    Connection,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import BN from 'borsh';
import bs58 from 'bs58';

import { readKeypairFromFile } from '../lib/readKeypairFromFile.js';

// mainnet
// const rpcUrl = 'https://solana-api.projectserum.com/';
// devnet
// const rpcUrl = 'https://api.devnet.solana.com';
// local
const rpcUrl = 'http://localhost:8899/';

// comm
const connection = new Connection(rpcUrl, 'finalized');
const idPath = '/home/ubuntu/solana_config/id.json';
const seed = 'mpAmmTest' + '0322';

// only at local
async function getPayer() {
    let keypair = await readKeypairFromFile(idPath);
    console.log('read payer', keypair.publicKey.toBase58());
    return keypair;
}

// test
async function main() {
    try {
        let args = process.argv.slice(2);
        let index = parseInt(args[0]);
        console.log('js start, index:', index);
        switch (index) {
            case 1:
                await getPayer();
                break;
            default:
                console.error('unknow index');
        }
        console.log('js end, index:', index);
    } catch (err) {
        console.error(err);
    }
}

main();
