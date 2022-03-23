import {
    PublicKey,
    Connection,
} from "@solana/web3.js";
import { readKeypairFromFile } from '../lib/readKeypairFromFile.js';
import {
    createMintAccount,
    createTokenAccount,
    createAssociatedTokenAccount,
    mintToTokenAccount
} from "../lib/tokenAccount.js";
import { AmmProgramId, createPoolAccount, findPool, findPoolByOwner, initPool, swap, Direction } from "../index.js";
import { getPoolData } from "../state.js";

// mainnet
// const rpcUrl = 'https://solana-api.projectserum.com/';
// devnet
// const rpcUrl = 'https://api.devnet.solana.com';
// local
const rpcUrl = 'http://localhost:8899/';

// comm
const connection = new Connection(rpcUrl, 'finalized');
const idPath = '/home/alex/.config/solana/id.json';
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
            console.log('create mint a', mintAKey);
        } else {
            console.error('create mint a error', res);
            return res;
        }
    }
    {
        let res = await createMintAccount(connection, wallet);
        if (res.code == 1) {
            mintBKey = res.data;
            console.log('create mint b', mintBKey);
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
            console.log('create user token a', userTokenAKey);
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
            console.log('create user token b', userTokenBKey);
        } else {
            console.error('create user token b error', res);
            return res;
        }
    }
    // mint token for user
    {
        let res = await mintToTokenAccount(connection, wallet, userTokenAKey, 3);
        if (res.code == 1) {
            console.log('mint to user token a ok');
        } else {
            console.error('mint to user token a error', res);
            return res;
        }
    }
    {
        let res = await mintToTokenAccount(connection, wallet, userTokenBKey, 1000);
        if (res.code == 1) {
            console.log('mint to user token b ok');
        } else {
            console.error('mint to user token b error', res);
            return res;
        }
    }
    // create fee mint
    {
        let res = await createMintAccount(connection, wallet);
        if (res.code == 1) {
            feeMintKey = res.data;
            console.log('create fee mint', feeMintKey);
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
            console.log('create user fee receiver ok', feeReceiver);
        } else {
            console.error('create user fee receiver error', res);
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
            // find payer owns pool
            let list = await findPoolByOwner(connection, payer.publicKey.toBase58());
            if (list.length > 0) {
                poolKey = list[0].pubkey.toBase58();
                console.log('pool exist', poolKey);
            }
        }
        if (poolKey == '') {
            // if pool is null, start init
            {
                // create mint user token account
                let res = await initEnv(connection, payer);
                if (res.code == 1) {
                    console.log('init env ok');
                } else {
                    console.error(res);
                    return res;
                }
            }
            {
                // create and init pool
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
                    seed,
                    feeParams,
                    1,
                    150,
                    255,
                    mintAKey,
                    mintBKey,
                );
                if (res.code == 1) {
                    poolKey = res.data;
                    console.log('init pool ok', res.data);
                } else {
                    console.error(res);
                    return res;
                }
            }
        }
        {
            // get pool data, check if the data is  right
            let res = await getPoolData(connection, poolKey);
            if (res.code == 1) {
                console.log('get pool data', res.data);
            }
        }
        {
            // swap a to b
            let res = await swap(connection, payer, poolKey, 1, Direction.A2B);
            if (res.code == 1) {
                console.log('swap a2b ok');
            }
        }
        {
            // swap b to a
            let res = await swap(connection, payer, poolKey, 1, Direction.B2A);
            if (res.code == 1) {
                console.log('swap b2a ok');
            }
        }
    } catch (err) {
        console.error(err);
    }
}

main();
