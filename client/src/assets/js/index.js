import {
  PublicKey,
  Transaction,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
} from "@solana/web3.js";
import { ChatingInstruction } from "./instruction.js";
import {
  PostDataLayout,
  getPostData,
  ContentLength,
  getPostDataRaw,
} from "./state.js";
import { signAndSendAllTransaction } from "./lib/sendTransction.js";
import bs58 from "bs58";
import { NullPublicKey } from "./lib/nullPublicKey.js";

// program
export const ChatingProgramId = "CRuMg5k6AyKkt9ed6iM4q43WDUsJwzdRn8ps6o1rkZD3";
const programId = new PublicKey(ChatingProgramId);
const seedPre = "Chat";

export async function post(connection, wallet, content = "") {
  if (!content || content === "") return { code: 0, msg: "null content" };
  // make buffer
  let buffers = string2Buffers(content);
  // make transactions
  let lamports = await connection.getMinimumBalanceForRentExemption(
    PostDataLayout.span
  );
  let txs = [];
  // create master post
  let masterKey;
  {
    let buffer0 = buffers[0];
    let res = await createPost(connection, wallet, {
      contentBuffer: buffer0,
      lamports,
    });
    if (res.code == 1) {
      masterKey = res.data.key;
      txs.push(res.data.tx);
    } else {
      return res;
    }
  }
  // append post
  if (buffers.length > 1) {
    for (let i = 1; i < buffers.length; i++) {
      let res = await createPost(connection, wallet, {
        masterKey,
        sharding: i,
        contentBuffer: buffers[i],
        lamports,
      });
      if (res.code == 1) {
        txs.push(res.data.tx);
      } else {
        return res;
      }
    }
  }
  let res = await signAndSendAllTransaction(connection, wallet, txs);
  console.log(res);
  return {
    code: 1,
    msg: "post ok",
    data: masterKey,
    signature: "",
  };
}

async function createPost(connection, wallet, params) {
  // use account
  let walletAcc = wallet.publicKey;
  let { masterKey, sharding, contentBuffer, lamports } = params;
  if (!masterKey) {
    masterKey = NullPublicKey;
    sharding = 0;
  }
  // create
  let seed = seedPre + new Date().getTime().toString();
  let postAcc = await PublicKey.createWithSeed(walletAcc, seed, programId);
  // check if exist
  let accountInfo = await connection.getAccountInfo(postAcc);
  if (accountInfo) {
    return { code: 2, msg: "post exist", data: postAcc.toBase58() };
  }
  // handle content
  if (contentBuffer.length > ContentLength) {
    return { code: 0, msg: "content too long" };
  }
  // make transaction
  let tx = new Transaction().add(
    SystemProgram.createAccountWithSeed({
      fromPubkey: walletAcc,
      basePubkey: walletAcc,
      newAccountPubkey: postAcc,
      seed,
      lamports,
      space: PostDataLayout.span,
      programId,
    }),
    ChatingInstruction.CreatePostShardingInstruction(
      sharding,
      new PublicKey(masterKey),
      contentBuffer,
      walletAcc,
      postAcc,
      SYSVAR_CLOCK_PUBKEY,
      programId
    )
  );
  return {
    code: 1,
    msg: "create post transaction",
    data: { key: postAcc.toBase58(), tx },
  };
}

export async function findAllPost(connection) {
  let config = {
    commitment: "finalized",
    filters: [{ dataSize: PostDataLayout.span }],
  };
  let list = await connection.getParsedProgramAccounts(programId, config);
  return list;
}

export async function findAllMasterPost(connection) {
  let config = {
    commitment: "finalized",
    filters: [
      { memcmp: { offset: 0, bytes: bs58.encode([1]) } },
      { dataSize: PostDataLayout.span },
    ],
  };
  let list = await connection.getParsedProgramAccounts(programId, config);
  return list.map((e) => {
    return getPostDataRaw(e.account.data, e.pubkey.toBase58());
  });
}

export async function findShardingPost(connection, masterKey) {
  let config = {
    commitment: "finalized",
    filters: [
      { memcmp: { offset: 0, bytes: bs58.encode([2]) } },
      { memcmp: { offset: 1 + 8 * 2 + 32, bytes: masterKey } },
      { dataSize: PostDataLayout.span },
    ],
  };
  let list = await connection.getParsedProgramAccounts(programId, config);
  return list.map((e) => {
    return getPostDataRaw(e.account.data, e.pubkey.toBase58());
  });
}

function string2Buffers(string = "") {
  // make buffer
  let buffer = Buffer.from(string);
  console.log(buffer);
  // splite buffer
  let count = Math.ceil(buffer.length / ContentLength);
  let buffers = new Array(count);
  for (let i = 0; i < count; i++) {
    buffers[i] = buffer.slice(i * ContentLength, (i + 1) * ContentLength);
  }
  // fill last one
  let lastBuffer = buffers[buffers.length - 1];
  if (lastBuffer.length < ContentLength) {
    console.log("last", lastBuffer);
    let lengthDiff = ContentLength - lastBuffer.length;
    let bufferDiff = new Array(lengthDiff);
    let newBuffer = Buffer.from(bufferDiff);
    lastBuffer = Buffer.concat([lastBuffer, newBuffer]);
    buffers[buffers.length - 1] = lastBuffer;
  }
  console.log(buffers);
  return buffers;
}

export { getPostData };
