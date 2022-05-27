import { TransactionInstruction } from "@solana/web3.js";
import bs58 from "bs58";
import * as BufferLayout from "@solana/buffer-layout";
import { ContentLength } from "./state.js";

// buffer layout
const CreatePostBuffer = BufferLayout.struct([
  BufferLayout.u8("i"),
  BufferLayout.blob(ContentLength, "content"),
]);
const CreatePostSharding = BufferLayout.struct([
  BufferLayout.u8("i"),
  BufferLayout.nu64("sharding"),
  BufferLayout.blob(32, "master_key"),
  BufferLayout.blob(ContentLength, "content"),
]);
const TerminateBuffer = BufferLayout.struct([BufferLayout.u8("i")]);

// instrucion
export class ChatingInstruction {
  static CreatePostInstruction(
    content,
    author_acc,
    post_acc,
    clock_acc,
    programId
  ) {
    console.log(
      "create post",
      "content",
      bs58.encode(content),
      "author_acc",
      author_acc.toBase58(),
      "post_acc",
      post_acc.toBase58(),
      "clock_acc",
      clock_acc.toBase58(),
      "program id",
      programId.toBase58()
    );
    // data
    let data = Buffer.alloc(CreatePostBuffer.span);
    CreatePostBuffer.encode(
      {
        i: 0,
        content,
      },
      data
    );
    // keys accounts
    let keys = [
      { pubkey: author_acc, isSigner: true, isWritable: false },
      { pubkey: post_acc, isSigner: false, isWritable: true },
      { pubkey: clock_acc, isSigner: false, isWritable: false },
    ];
    // make instruction
    return new TransactionInstruction({ keys, programId, data });
  }
  static CreatePostShardingInstruction(
    sharding,
    master_acc,
    content,
    author_acc,
    post_acc,
    clock_acc,
    programId
  ) {
    console.log(
      "create post sharding",
      "sharding",
      sharding,
      "master_acc",
      master_acc.toBase58(),
      "content",
      bs58.encode(content),
      "author_acc",
      author_acc.toBase58(),
      "post_acc",
      post_acc.toBase58(),
      "clock_acc",
      clock_acc.toBase58(),
      "program id",
      programId.toBase58()
    );
    // data
    let data = Buffer.alloc(CreatePostSharding.span);
    CreatePostSharding.encode(
      {
        i: 1,
        sharding,
        master_key: master_acc.toBytes(),
        content,
      },
      data
    );
    // keys accounts
    let keys = [
      { pubkey: author_acc, isSigner: true, isWritable: false },
      { pubkey: post_acc, isSigner: false, isWritable: true },
      { pubkey: clock_acc, isSigner: false, isWritable: false },
    ];
    // make instruction
    return new TransactionInstruction({ keys, programId, data });
  }
  static TerminateInstruction(author_acc, post_acc, programId) {
    console.log(
      "terminate",
      "author_acc",
      author_acc,
      "post_acc",
      post_acc,
      "program id",
      programId.toBase58()
    );
    // data
    let data = Buffer.alloc(TerminateBuffer.span);
    TerminateBuffer.encode(
      {
        i: 10,
      },
      data
    );
    // keys accounts
    let keys = [
      { pubkey: author_acc, isSigner: true, isWritable: true },
      { pubkey: post_acc, isSigner: false, isWritable: true },
    ];
    // make instruction
    return new TransactionInstruction({ keys, programId, data });
  }
}
