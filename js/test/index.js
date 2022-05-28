import { Connection, PublicKey } from "@solana/web3.js";
import {
  createPool,
  Direction,
  findPoolByOwner,
  swap,
  terminate,
  withdrawalFee,
} from "../index.js";
import { readKeypairFromFile } from "../lib/readKeypairFromFile.js";
import { getMintData, initMintAndTokenAccount } from "../lib/tokenAccount.js";
import { getPoolData } from "../state.js";

// mainnet
// const rpcUrl = 'https://solana-api.projectserum.com/';
// devnet
// const rpcUrl = 'https://api.devnet.solana.com';
// local
const rpcUrl = "http://localhost:8899/";

// comm
const connection = new Connection(rpcUrl, "finalized");
const idPath = "/home/alex/.config/solana/id.json";
const seed = "mpAmmTest" + "0322";

// key
let mintAKey = "";
let mintBKey = "";
let feeMintKey = "";
let poolKey = "";
let feeReceiver = "";

async function getPayer() {
  let keypair = await readKeypairFromFile(idPath);
  console.log("read payer", keypair.publicKey.toBase58());
  return keypair;
}

async function initEnv(connection, wallet) {
  // create mint
  {
    let res = await initMintAndTokenAccount(connection, wallet, 9, 1000);
    if (res.code == 1) {
      mintAKey = res.data;
      console.log("mint a", mintAKey);
    } else {
      return res;
    }
  }
  {
    let res = await initMintAndTokenAccount(connection, wallet, 6, 1000);
    if (res.code == 1) {
      mintBKey = res.data;
      console.log("mint b", mintBKey);
    } else {
      return res;
    }
  }
  return { code: 1, msg: "init env ok" };
}

export async function init() {
  try {
    let payer = await getPayer();
    // create mint user token account
    let res = await initEnv(connection, payer);
    if (res.code == 1) {
      console.log("init env ok");
    } else {
      console.error(res);
      return res;
    }
  } catch (err) {
    console.error("init error", err);
  }
}

// test
export async function main() {
  try {
    let payer = await getPayer();
    {
      // find payer owns pool
      let list = await findPoolByOwner(connection, payer.publicKey.toBase58());
      if (list.length > 0) {
        poolKey = list[0].pubkey.toBase58();
        console.log("pool exist", poolKey);
      }
    }
    if (poolKey == "") {
      // if pool is null, start init
      {
        // create mint user token account
        let res = await initEnv(connection, payer);
        if (res.code == 1) {
          console.log("init env ok");
        } else {
          console.error(res);
          return res;
        }
      }
      {
        // create and init pool
        // 0.01 means 1%
        // fee mint must be mint b
        let feeParams = {
          rate: 0.0045,
          mint: new PublicKey(mintBKey),
        };
        let res = await createPool(
          connection,
          payer,
          feeParams,
          1,
          150,
          255,
          mintAKey,
          mintBKey
        );
        if (res.code == 1) {
          poolKey = res.data;
          console.log("init pool ok", res.data);
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
        console.log("get pool data", res.data);
      } else {
        console.error(res);
        return;
      }
    }
    {
      // swap a to b
      let res = await swap(connection, payer, poolKey, 1, Direction.A2B);
      if (res.code == 1) {
        console.log("swap a2b ok");
      } else {
        console.error(res);
        return;
      }
    }
    {
      // swap b to a
      let res = await swap(connection, payer, poolKey, 1, Direction.B2A);
      if (res.code == 1) {
        console.log("swap b2a ok");
      } else {
        console.error(res);
        return;
      }
    }
    {
      // withdrawal fee
      let res = await withdrawalFee(connection, payer, poolKey);
      if (res.code == 1) {
        console.log("withdrawal fee ok");
      } else {
        console.error(res);
        return;
      }
    }
    {
      // terminate
      let res = await terminate(connection, payer, poolKey);
      if (res.code == 1) {
        console.log("terminate ok");
      } else {
        console.error(res);
        return;
      }
    }
  } catch (err) {
    console.error(err);
  }
}
