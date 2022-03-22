import { TransactionInstruction } from "@solana/web3.js";
import * as BufferLayout from 'buffer-layout';

// buffer layout
const InitBuffer = BufferLayout.struct([
    BufferLayout.u8('i'),
    BufferLayout.u8('nonce'),
    BufferLayout.nu64('fee_1'),
    BufferLayout.nu64('fee_2'),
    BufferLayout.nu64('fee_3'),
    BufferLayout.nu64('fee_4'),
    BufferLayout.nu64('fee_5'),
    BufferLayout.nu64('amount_a'),
    BufferLayout.nu64('amount_b'),
    BufferLayout.nu64('tolerance'),
]);
const UpdatePoolBuffer = BufferLayout.struct([
    BufferLayout.u8('i'),
]);
const UpdateStatusBuffer = BufferLayout.struct([
    BufferLayout.u8('i'),
    BufferLayout.u8('status'),
]);
const UpdateToleranceBuffer = BufferLayout.struct([
    BufferLayout.u8('i'),
    BufferLayout.nu64('tolerance'),
]);
const SwapBuffer = BufferLayout.struct([
    BufferLayout.u8('i'),
    BufferLayout.nu64('amount'),
    BufferLayout.u8('direction'),
]);

// instrucion
export class AmmInstruction {
    static createInitInstruction(
        nonce,
        fee_1,
        fee_2,
        fee_3,
        fee_4,
        fee_5,
        amount_a,
        amount_b,
        pool_acc,
        owner_acc,
        mint_a_acc,
        mint_b_acc,
        vault_a_acc,
        vault_b_acc,
        fee_vault_acc,
        fee_receiver_1_acc,
        fee_receiver_2_acc,
        fee_receiver_3_acc,
        fee_receiver_4_acc,
        fee_receiver_5_acc,
        fee_mint_acc,
        pool_pda,
        owner_token_a_acc,
        owner_token_b_acc,
        token_program_acc,
        programId,
    ) {
        console.log(
            'init',
            'nonce', nonce,
            'fee_1', fee_1,
            'fee_2', fee_2,
            'fee_3', fee_3,
            'fee_4', fee_4,
            'fee_5', fee_5,
            'amount_a', amount_a,
            'amount_b', amount_b,
            'pool_acc', pool_acc.toBase58(),
            'owner_acc', owner_acc.toBase58(),
            'mint_a_acc', mint_a_acc.toBase58(),
            'mint_b_acc', mint_b_acc.toBase58(),
            'vault_a_acc', vault_a_acc.toBase58(),
            'vault_b_acc', vault_b_acc.toBase58(),
            'fee_vault_acc', fee_vault_acc.toBase58(),
            'fee_receiver_1_acc', fee_receiver_1_acc.toBase58(),
            'fee_receiver_2_acc', fee_receiver_2_acc.toBase58(),
            'fee_receiver_3_acc', fee_receiver_3_acc.toBase58(),
            'fee_receiver_4_acc', fee_receiver_4_acc.toBase58(),
            'fee_receiver_5_acc', fee_receiver_5_acc.toBase58(),
            'fee_mint_acc', fee_mint_acc.toBase58(),
            'pool_pda', pool_pda.toBase58(),
            'owner_token_a_acc', owner_token_a_acc.toBase58(),
            'owner_token_b_acc', owner_token_b_acc.toBase58(),
            'token_program_acc', token_program_acc.toBase58(),
            'program id', programId.toBase58()
        );
        // data
        let data = Buffer.alloc(InitBuffer.span);
        InitBuffer.encode({
            i: 0,
            nonce,
            fee_1,
            fee_2,
            fee_3,
            fee_4,
            fee_5,
            amount_a,
            amount_b,
        }, data);
        // keys accounts
        let keys = [
            { pubkey: pool_acc, isSigner: false, isWritable: true },
            { pubkey: owner_acc, isSigner: true, isWritable: false },
            { pubkey: mint_a_acc, isSigner: false, isWritable: false },
            { pubkey: mint_b_acc, isSigner: false, isWritable: false },
            { pubkey: vault_a_acc, isSigner: false, isWritable: true },
            { pubkey: vault_b_acc, isSigner: false, isWritable: true },
            { pubkey: fee_vault_acc, isSigner: false, isWritable: false },
            { pubkey: fee_receiver_1_acc, isSigner: false, isWritable: false },
            { pubkey: fee_receiver_2_acc, isSigner: false, isWritable: false },
            { pubkey: fee_receiver_3_acc, isSigner: false, isWritable: false },
            { pubkey: fee_receiver_4_acc, isSigner: false, isWritable: false },
            { pubkey: fee_receiver_5_acc, isSigner: false, isWritable: false },
            { pubkey: fee_mint_acc, isSigner: false, isWritable: false },
            { pubkey: pool_pda, isSigner: false, isWritable: false },
            { pubkey: owner_token_a_acc, isSigner: false, isWritable: true },
            { pubkey: owner_token_b_acc, isSigner: false, isWritable: true },
            { pubkey: token_program_acc, isSigner: false, isWritable: false },
        ];
        // make instruction
        let instrucion = new TransactionInstruction({ keys, programId, data });
        return instrucion;
    }
    static createUpdatePoolInstruction(
        pool_acc,
        owner_acc,
        fee_receiver_1_acc,
        fee_receiver_2_acc,
        fee_receiver_3_acc,
        fee_receiver_4_acc,
        fee_receiver_5_acc,
        programId,
    ) {
        console.log(
            'update pool',
            'pool_acc', pool_acc.toBase58(),
            'owner_acc', owner_acc.toBase58(),
            'fee_receiver_1_acc', fee_receiver_1_acc.toBase58(),
            'fee_receiver_2_acc', fee_receiver_2_acc.toBase58(),
            'fee_receiver_3_acc', fee_receiver_3_acc.toBase58(),
            'fee_receiver_4_acc', fee_receiver_4_acc.toBase58(),
            'fee_receiver_5_acc', fee_receiver_5_acc.toBase58(),
            'program id', programId.toBase58()
        );
        // data
        let data = Buffer.alloc(UpdatePoolBuffer.span);
        UpdatePoolBuffer.encode({
            i: 1,
        }, data);
        // keys accounts
        let keys = [
            { pubkey: pool_acc, isSigner: false, isWritable: true },
            { pubkey: owner_acc, isSigner: true, isWritable: false },
            { pubkey: fee_receiver_1_acc, isSigner: false, isWritable: false },
            { pubkey: fee_receiver_2_acc, isSigner: false, isWritable: false },
            { pubkey: fee_receiver_3_acc, isSigner: false, isWritable: false },
            { pubkey: fee_receiver_4_acc, isSigner: false, isWritable: false },
            { pubkey: fee_receiver_5_acc, isSigner: false, isWritable: false },
        ];
        // make instruction
        let instrucion = new TransactionInstruction({ keys, programId, data });
        return instrucion;
    }
    static createUpdateStatusInstrucion(
        status,
        pool_acc,
        owner_acc,
        programId,
    ) {
        console.log(
            'update status',
            'status', status,
            'pool_acc', pool_acc.toBase58(),
            'owner_acc', owner_acc.toBase58(),
            'program id', programId.toBase58()
        );
        // data
        let data = Buffer.alloc(UpdateStatusBuffer.span);
        UpdateStatusBuffer.encode({
            i: 2,
            status,
        }, data);
        // keys accounts
        let keys = [
            { pubkey: pool_acc, isSigner: false, isWritable: true },
            { pubkey: owner_acc, isSigner: true, isWritable: false },
        ];
        // make instruction
        let instrucion = new TransactionInstruction({ keys, programId, data });
        return instrucion;
    }
    static createUpdateToleranceInstruction(
        tolerance,
        pool_acc,
        owner_acc,
        programId,
    ) {
        console.log(
            'update tolerance',
            'tolerance', tolerance,
            'pool_acc', pool_acc.toBase58(),
            'owner_acc', owner_acc.toBase58(),
            'program id', programId.toBase58()
        );
        // data
        let data = Buffer.alloc(UpdateToleranceBuffer.span);
        UpdateToleranceBuffer.encode({
            i: 3,
            tolerance,
        }, data);
        // keys accounts
        let keys = [
            { pubkey: pool_acc, isSigner: false, isWritable: true },
            { pubkey: owner_acc, isSigner: true, isWritable: false },
        ];
        // make instruction
        let instrucion = new TransactionInstruction({ keys, programId, data });
        return instrucion;
    }
    static createSwapInstrucion(
        amount,
        direction,
        pool_acc,
        vault_a_acc,
        vault_b_acc,
        fee_vault,
        pool_pda,
        user_wallet_acc,
        user_token_a_acc,
        user_token_b_acc,
        user_fee_acc,
        token_program_acc,
        programId,
    ) {
        console.log(
            'swap',
            'amount', amount,
            'direction', direction,
            'pool_acc', pool_acc.toBase58(),
            'vault_a_acc', vault_a_acc.toBase58(),
            'vault_b_acc', vault_b_acc.toBase58(),
            'fee_vault', fee_vault.toBase58(),
            'pool_pda', pool_pda.toBase58(),
            'user_wallet_acc', user_wallet_acc.toBase58(),
            'user_token_a_acc', user_token_a_acc.toBase58(),
            'user_token_b_acc', user_token_b_acc.toBase58(),
            'user_fee_acc', user_fee_acc.toBase58(),
            'token_program_acc', token_program_acc.toBase58(),
            'program id', programId.toBase58()
        );
        // data
        let data = Buffer.alloc(SwapBuffer.span);
        SwapBuffer.encode({
            i: 10,
            amount,
            direction,
        }, data);
        // keys accounts
        let keys = [
            { pubkey: pool_acc, isSigner: false, isWritable: true },
            { pubkey: vault_a_acc, isSigner: false, isWritable: true },
            { pubkey: vault_b_acc, isSigner: false, isWritable: true },
            { pubkey: fee_vault, isSigner: false, isWritable: true },
            { pubkey: pool_pda, isSigner: false, isWritable: false },
            { pubkey: user_wallet_acc, isSigner: true, isWritable: false },
            { pubkey: user_token_a_acc, isSigner: false, isWritable: true },
            { pubkey: user_token_b_acc, isSigner: false, isWritable: true },
            { pubkey: user_fee_acc, isSigner: false, isWritable: true },
            { pubkey: token_program_acc, isSigner: false, isWritable: false },
        ];
        // make instruction
        let instrucion = new TransactionInstruction({ keys, programId, data });
        return instrucion;
    }
}
