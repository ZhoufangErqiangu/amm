import {
    Keypair,
    PublicKey,
    Connection,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import BN from 'borsh';
import bs58 from 'bs58';

import { readKeypairFromFile } from '../lib/readKeypairFromFile.js';
import { createMintAccount, createTokenAccount, mintToTokenAccount } from "../lib/tokenAccount.js";
import { createPoolAccount, initPool } from "../index.js";

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

// key
let mintAKey = '';
let mintBKey = '';
let feeMintKey = '';
let poolKey = '';
let feeReceiver = '';

async function getPayer() {
    let keypair = await readKeypairFromFile(idPath);
    console.log('read payer', keypair.publicKey.toBase58());
    return keypair;
}

async function initEnv(connection, wallet) {
    // create mint
    {
        // mint a is nft, its decimal is zero.
        let res = await createMintAccount(connection, wallet, 0);
        if (res.code == 1) {
            mintAKey = res.data;
        } else {
            console.error('create mint a error', res);
            return res;
        }
    }
    {
        let res = await createMintAccount(connection, wallet);
        if (res.code == 1) {
            mintBKey = res.data;
        } else {
            console.error('create mint b error', res);
            return res;
        }
    }
    // create user token account
    let userTokenAKey;
    {
        let res = await createTokenAccount(connection, wallet, mintAKey);
        if (res.code == 1) {
            userTokenAKey = res.data;
        } else {
            console.error('create user token a error', res);
            return res;
        }
    }
    let userTokenBKey;
    {
        let res = await createTokenAccount(connection, wallet, mintBKey);
        if (res.code == 1) {
            userTokenBKey = res.data;
        } else {
            console.error('create user token b error', res);
            return res;
        }
    }
    // mint token for user
    {
        let res = await mintToTokenAccount(connection, wallet, userTokenAKey, 3);
        if (res.code != 1) {
            console.error('mint to user token a error', res);
            return res;
        }
    }
    {
        let res = await mintToTokenAccount(connection, wallet, userTokenBKey, 1000);
        if (res.code != 1) {
            console.error('mint to user token b error', res);
            return res;
        }
    }
    // create fee mint
    {
        let res = await createMintAccount(connection, wallet);
        if (res.code == 1) {
            feeMintKey = res.data;
        } else {
            console.error('create fee mint error', res);
            return res;
        }
    }
    // create fee receiver
    {
        let res = await createTokenAccount(connection, wallet, feeMintKey);
        if (res.code == 1) {
            feeReceiver = res.data;
        } else {
            console.error('create user token a error', res);
            return res;
        }
    }
    return { code: 1, msg: 'init env ok' };
}

// test
async function main() {
    try {
        let payer = await getPayer();
        {
            let res = await initEnv(connection, payer);
            if (res.code == 1) {
                console.log('init env ok');
            } else {
                return res;
            }
        }
        {
            let res = await createPoolAccount(connection, payer, seed);
            if (res.code == 1) {
                poolKey = res.data;
                console.log('create pool ok', poolKey);
            } else {
                return res;
            }
        }
        {
            // 0.01 means 1%
            let feeParams = {
                // Liquidity Providers
                rate1: 0.002,
                // Mercanti Stakers
                rate2: 0.0005,
                // Project / DAO
                rate3: 0.0015,
                // $MARCO Buy-Back & Burn
                rate4: 0.0005,
                // reserved
                rate5: 0,
                // Liquidity Providers
                receiver1: new PublicKey(feeReceiver),
                // Mercanti Stakers
                receiver2: new PublicKey(feeReceiver),
                // Project / DAO
                receiver3: new PublicKey(feeReceiver),
                // $MARCO Buy-Back & Burn
                receiver4: new PublicKey(feeReceiver),
                // reserved
                receiver5: new PublicKey(feeReceiver),
                // ?must be marco?
                mint: new PublicKey(feeMintKey),
            }
            let res = await initPool(
                connection,
                payer,
                poolKey,
                feeParams,
                1,
                150,
                mintAKey,
                mintBKey,
            );
            if (res.code == 1) {
                console.log('init pool ok');
            } else {
                return res;
            }
        }

    } catch (err) {
        console.error(err);
    }
}

main();
