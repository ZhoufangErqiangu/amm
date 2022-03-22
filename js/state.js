import { PublicKey } from '@solana/web3.js';
import * as BufferLayout from 'buffer-layout';

// buffer layout
export const PoolDataLayout = BufferLayout.struct([
    BufferLayout.u8("status"),
    BufferLayout.u8("nonce"),
    BufferLayout.nu64("ka"),
    BufferLayout.nu64("kb"),
    BufferLayout.nu64("tolerance"),
    BufferLayout.f64("fee_1"),
    BufferLayout.f64("fee_2"),
    BufferLayout.f64("fee_3"),
    BufferLayout.f64("fee_4"),
    BufferLayout.f64("fee_5"),
    BufferLayout.blob(32, "owner"),
    BufferLayout.blob(32, "mint_a"),
    BufferLayout.blob(32, "mint_b"),
    BufferLayout.blob(32, "vault_a"),
    BufferLayout.blob(32, "vault_b"),
    BufferLayout.blob(32, "fee_vault"),
    BufferLayout.blob(32, "fee_receiver_1"),
    BufferLayout.blob(32, "fee_receiver_2"),
    BufferLayout.blob(32, "fee_receiver_3"),
    BufferLayout.blob(32, "fee_receiver_4"),
    BufferLayout.blob(32, "fee_receiver_5"),
    BufferLayout.blob(32, "fee_mint"),
]);

// function
export async function getPoolData(connection, poolKey) {
    // use account
    let poolAcc = new PublicKey(poolKey);
    // get data
    let poolData = await connection.getAccountInfo(poolAcc);
    if (poolData) {
        let temp = PoolDataLayout.decode(poolData.data);
        temp['poolKey'] = poolKey;
        return { code: 1, msg: 'get pool data ok', data: handleKey(temp) };
    } else {
        return { code: 0, msg: 'pool is null', data: null };
    }
}

function handleKey(data) {
    for (let key in data) {
        if (data[key].length == 32) {
            let pubkey = new PublicKey(data[key]);
            data[key] = pubkey.toBase58();
        }
    }
    return data;
}
