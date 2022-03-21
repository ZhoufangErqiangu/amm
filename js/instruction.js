import { TransactionInstruction } from "@solana/web3.js";
import * as BufferLayout from 'buffer-layout';

// buffer layout
const initBuffer = BufferLayout.struct([
    BufferLayout.u8('i'),
    BufferLayout.u8('nonce'),
]);

// instrucion
export class AmmInstruction {
    static createInitInstruction(
        nonce,
        registrar_acc,
        programId,
    ) {
        console.log(
            'init',
            'nonce', nonce,
            'registrar', registrar_acc.toBase58(),
            'program id', programId.toBase58()
        );
        // data
        let data = Buffer.alloc(initBuffer.span);
        initBuffer.encode({
            i: 0,
            nonce: nonce,
        }, data);
        // keys accounts
        let keys = [
            { pubkey: registrar_acc, isSigner: false, isWritable: true },
        ];
        // make instruction
        let instrucion = new TransactionInstruction({ keys, programId, data });
        return instrucion;
    }
}
