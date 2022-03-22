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
        return { code: 1, msg: 'pool create ok', data: poolAcc.toBase58() };
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
    // make transaction
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
        feeParams.rate1,
        feeParams.rate2,
        feeParams.rate3,
        feeParams.rate4,
        feeParams.rate5,
        amountA * 10 ** mintAData.decimals,
        amountB * 10 ** mintBData.decimals,
        poolAcc,
        walletAcc,
        mintAAcc,
        mintBAcc,
        vaultAAccount.publicKey,
        vaultBAccount.publicKey,
        feeVaultAccount.publicKey,
        feeParams.receiver1,
        feeParams.receiver2,
        feeParams.receiver3,
        feeParams.receiver4,
        feeParams.receiver5,
        feeParams.mint,
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
        return { code: 1, msg: 'init pool ok', data: poolAcc.toBase58() };
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

export { getPoolData };
