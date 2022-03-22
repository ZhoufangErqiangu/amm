import {
    Keypair,
    PublicKey,
    Transaction,
    SystemProgram,
} from "@solana/web3.js";
import { AccountLayout, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { AmmInstruction } from './instruction.js';
import { PoolDataLayout, getPoolData } from "./state.js";
import { signAndSendTransaction } from "./lib/sendTransction.js";
import { getMintData, getTokenAccountMaxAmount } from "./lib/tokenAccount.js";

// program
export const AmmProgramId = 'aAmLZ9yP1adeZyRC9qMskX9e1Ma2gR4ktpyrDCWPkdm';
const programId = new PublicKey(AmmProgramId);

const PercenMul = 10 ** 6;

export async function createPoolAccount(connection, wallet, seed) {
    // use account
    let walletAcc = wallet.publicKey;
    // create
    let poolAcc = await PublicKey.createWithSeed(walletAcc, seed, programId);
    // check if exist
    let poolData = await connection.getAccountInfo(poolAcc);
    if (poolData) {
        return { code: 2, msg: 'pool exist', data: poolAcc.toBase58() };
    }
    // make transaction
    let lamports = await connection.getMinimumBalanceForRentExemption(PoolDataLayout.span);
    let tx = new Transaction().add(SystemProgram.createAccountWithSeed({
        fromPubkey: walletAcc,
        newAccountPubkey: poolAcc,
        seed,
        lamports,
        space: PoolDataLayout.span,
        programId
    }));
    let res = await signAndSendTransaction(connection, wallet, null, tx);
    if (res.code == 1) {
        return { code: 1, msg: 'pool create ok', data: poolAcc.toBase58(), signature: res.data };
    } else {
        return res;
    }
}

export async function initPool(connection, wallet, poolKey, feeParams, amountA, amountB, mintAKey, mintBKey) {
    // use account
    let walletAcc = wallet.publicKey;
    let poolAcc = new PublicKey(poolKey);
    let [poolPDA, nonce] = await PublicKey.findProgramAddress([poolAcc.toBuffer()], programId);
    let mintAAcc = new PublicKey(mintAKey);
    let mintBAcc = new PublicKey(mintBKey);
    let userTokenAAcc;
    {
        let res = await getTokenAccountMaxAmount(connection, wallet, mintAKey);
        if (res.code == 1) {
            userTokenAAcc = res.data.publicKey;
        } else {
            return res;
        }
    }
    let userTokenBAcc;
    {
        let res = await getTokenAccountMaxAmount(connection, wallet, mintBKey);
        if (res.code == 1) {
            userTokenBAcc = res.data.publicKey;
        } else {
            return res;
        }
    }
    // use data
    let mintAData;
    {
        let res = await getMintData(connection, mintAKey);
        if (res.code == 1) {
            mintAData = res.data;
        } else {
            return res;
        }
    }
    let mintBData;
    {
        let res = await getMintData(connection, mintBKey);
        if (res.code == 1) {
            mintBData = res.data;
        } else {
            return res;
        }
    }
    // create account
    let lamports = await connection.getMinimumBalanceForRentExemption(AccountLayout.span);
    let vaultAAccount = new Keypair();
    let vaultBAccount = new Keypair();
    let feeVaultAccount = new Keypair();
    // make transaction
    let tx = new Transaction().add(SystemProgram.createAccount({
        fromPubkey: walletAcc,
        newAccountPubkey: vaultAAccount,
        lamports,
        space: AccountLayout.span,
        programId: TOKEN_PROGRAM_ID,
    }), Token.createInitAccountInstruction(
        TOKEN_PROGRAM_ID,
        mintAAcc,
        vaultAAccount.publicKey,
        poolPDA,
    ), SystemProgram.createAccount({
        fromPubkey: walletAcc,
        newAccountPubkey: vaultBAccount,
        lamports,
        space: AccountLayout.span,
        programId: TOKEN_PROGRAM_ID,
    }), Token.createInitAccountInstruction(
        TOKEN_PROGRAM_ID,
        mintBAcc,
        vaultBAccount.publicKey,
        poolPDA,
    ), SystemProgram.createAccount({
        fromPubkey: walletAcc,
        newAccountPubkey: feeVaultAccount,
        lamports,
        space: AccountLayout.span,
        programId: TOKEN_PROGRAM_ID,
    }), Token.createInitAccountInstruction(
        TOKEN_PROGRAM_ID,
        feeParams.mint,
        feeVaultAccount.publicKey,
        poolPDA,
    ), AmmInstruction.createInitInstruction(
        nonce,
        feeParams.rate1 * 10 ** PercenMul,
        feeParams.rate2 * 10 ** PercenMul,
        feeParams.rate3 * 10 ** PercenMul,
        feeParams.rate4 * 10 ** PercenMul,
        feeParams.rate5 * 10 ** PercenMul,
        amountA * 10 ** mintAData.decimals,
        amountB * 10 ** mintBData.decimals,
        poolAcc,
        walletAcc,
        mintAAcc,
        mintBAcc,
        vaultAAccount.publicKey,
        vaultBAccount.publicKey,
        feeVaultAccount.publicKey,
        new PublicKey(feeParams.receiver1),
        new PublicKey(feeParams.receiver2),
        new PublicKey(feeParams.receiver3),
        new PublicKey(feeParams.receiver4),
        new PublicKey(feeParams.receiver5),
        new PublicKey(feeParams.mint),
        poolPDA,
        userTokenAAcc,
        userTokenBAcc,
        TOKEN_PROGRAM_ID,
        programId,
    ));
    let res = await signAndSendTransaction(connection, wallet, [
        vaultAAccount,
        vaultBAccount,
        feeVaultAccount,
    ], tx);
    if (res.code == 1) {
        return { code: 1, msg: 'init pool ok', data: poolAcc.toBase58(), signature: res.data };
    } else {
        return res;
    }
}

export async function getPoolPDA(connection, poolKey) {
    // use account
    let poolAcc = new PublicKey(poolKey);
    // get data
    let poolData;
    {
        let res = await getPoolData(connection, poolKey);
        if (res.code == 1) {
            poolData = res.data;
        } else {
            return res;
        }
    }
    // create pda
    let poolPDA = await PublicKey.createProgramAddress([
        poolAcc.toBuffer(),
        Buffer.from([poolData.nonce]),
    ]);
    return { code: 1, msg: 'get pda ok', data: poolPDA };
}

export async function findPoolByOwner(connection, ownerKey) {
    let config = {
        commitment: 'finalized',
        filters: [
            { memcmp: { offset: 1 * 2 + 8 * 8, bytes: ownerKey } },
            { dataSize: PoolDataLayout.span },
        ],
    };
    let list = await connection.getParsedProgramAccounts(programId, config);
    return list;
}

export async function findPoolByMints(connection, mintAKey, mintBKey) {
    let config = {
        commitment: 'finalized',
        filters: [
            { memcmp: { offset: 1 * 2 + 8 * 8 + 32, bytes: mintAKey } },
            { memcmp: { offset: 1 * 2 + 8 * 8 + 32 * 2, bytes: mintBKey } },
            { dataSize: PoolDataLayout.span },
        ],
    };
    let list = await connection.getParsedProgramAccounts(programId, config);
    return list;
}

export async function updatePool(connection, wallet, poolKey, feeParams) {
    // use account
    let walletAcc = wallet.publicKey;
    let poolAcc = new PublicKey(poolKey);
    // make transaction
    let tx = new Transaction().add(AmmInstruction.createUpdatePoolInstruction(
        poolAcc,
        walletAcc,
        new PublicKey(feeParams.receiver1),
        new PublicKey(feeParams.receiver2),
        new PublicKey(feeParams.receiver3),
        new PublicKey(feeParams.receiver4),
        new PublicKey(feeParams.receiver5),
        programId,
    ));
    let res = await signAndSendTransaction(connection, wallet, null, tx);
    if (res.code == 1) {
        return { code: 1, msg: 'update pool ok', data: poolAcc.toBase58(), signature: res.data };
    } else {
        return res;
    }
}

export async function updateStatus(connection, wallet, poolKey, status) {
    // use account
    let walletAcc = wallet.publicKey;
    let poolAcc = new PublicKey(poolKey);
    // make transaction
    let tx = new Transaction().add(AmmInstruction.createUpdateStatusInstrucion(
        status,
        poolAcc,
        walletAcc,
        programId,
    ));
    let res = await signAndSendTransaction(connection, wallet, null, tx);
    if (res.code == 1) {
        return { code: 1, msg: 'update status ok', data: poolAcc.toBase58(), signature: res.data };
    } else {
        return res;
    }
}

export async function updateTolerance(connection, wallet, poolKey, tolerance) {
    // use account
    let walletAcc = wallet.publicKey;
    let poolAcc = new PublicKey(poolKey);
    // make transaction
    let tx = new Transaction().add(AmmInstruction.createUpdateToleranceInstruction(
        tolerance,
        poolAcc,
        walletAcc,
        programId,
    ));
    let res = await signAndSendTransaction(connection, wallet, null, tx);
    if (res.code == 1) {
        return { code: 1, msg: 'update tolerance ok', data: poolAcc.toBase58(), signature: res.data };
    } else {
        return res;
    }
}

// 1 is a2b, 2 is b2a
export async function swap(connection, wallet, poolKey, amount, direction) {
    // use account
    let walletAcc = wallet.publicKey;
    let poolAcc = new PublicKey(poolKey);
    // use data
    let poolData;
    {
        let res = await getPoolData(connection, poolKey);
        if (res.code == 1) {
            poolData = res.data;
        } else {
            return res;
        }
    }
    let mintAData;
    {
        let res = await getMintData(connection, poolData.mint_a);
        if (res.code == 1) {
            mintAData = res.data;
        } else {
            return res;
        }
    }
    // use account
    let poolPDA;
    {
        let res = await getPoolPDA(connection, poolKey);
        if (res.code == 1) {
            poolPDA = res.data;
        } else {
            return res;
        }
    }
    let userTokenAKey;
    
    // make transaction
    let tx = new Transaction().add(AmmInstruction.createSwapInstrucion(
        amount * 10 ** mintAData.decimals,
        direction,
        poolAcc,
        new PublicKey(poolData.vault_a),
        new PublicKey(poolData.vault_b),
        new PublicKey(poolData.fee_vault),
        poolPDA,
        walletAcc,
    ));
}

export { getPoolData };
