import { PublicKey } from "@solana/web3.js";
import * as BufferLayout from "@solana/buffer-layout";
import bs58 from "bs58";

export const ContentLength = 512;

// buffer layout
export const PostDataLayout = BufferLayout.struct([
  BufferLayout.u8("postType"),
  BufferLayout.nu64("sharding"),
  BufferLayout.ns64("time"),
  BufferLayout.blob(32, "author"),
  BufferLayout.blob(32, "master"),
  BufferLayout.blob(32, "quote"),
  BufferLayout.blob(ContentLength, "content"),
]);

// enum
export class PostType {
  static option = [
    { value: 1, label: "Master" },
    { value: 2, label: "MasterSharding" },
    { value: 3, label: "Reply" },
    { value: 4, label: "ReplySharding" },
  ];
  static getLabel(value = 1) {
    let postType = PostType.option.find((e) => {
      return e.value === value;
    });
    return postType ? postType.label : PostType[0].label;
  }
  static getValue(label = "") {
    let postType = PostType.option.find((e) => {
      return e.label === label;
    });
    return postType ? postType.value : PostType[0].value;
  }
}

// function
export async function getPostData(connection, postKey) {
  // use account
  let postAcc = new PublicKey(postKey);
  // get data
  let postData = await connection.getAccountInfo(postAcc);
  if (postData) {
    let temp = PostDataLayout.decode(postData.data);
    temp.key = postKey;
    temp.time *= 1000;
    temp.postTypeLabel = PostType.getLabel(temp.postType);
    temp.content = handleContent(temp.content);
    return { code: 1, msg: "get post data ok", data: handleKey(temp) };
  } else {
    return { code: 0, msg: "post is null", data: null };
  }
}

export function getPostDataRaw(data, key) {
  let temp = PostDataLayout.decode(data);
  temp.key = key;
  temp.time *= 1000;
  temp.postTypeLabel = PostType.getLabel(temp.postType);
  temp.content = handleContent(temp.content);
  return handleKey(temp);
}

function handleContent(content) {
  let buffer = Buffer.from(content);
  // let string = buffer.toString();
  // string = string.replace("\x00", "");
  return buffer.toString();
}

function handleKey(data) {
  for (let key in data) {
    if (data[key].length == 32) {
      data[key] = bs58.encode(data[key]);
    }
  }
  return data;
}
