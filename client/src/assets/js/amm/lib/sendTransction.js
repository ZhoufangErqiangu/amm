export async function signAndSendTransaction(connection, wallet, partialSignerList, transaction) {
  // use account
  let walletAcc = wallet.publicKey;
  // make transaction
  transaction.feePayer = walletAcc;
  let { blockhash } = await connection.getRecentBlockhash();
  transaction.recentBlockhash = blockhash;
  // sign
  let signed;
  try {
    if (wallet.secretKey) {
      // only at local
      transaction.partialSign(wallet);
      signed = transaction;
    } else {
      signed = await wallet.signTransaction(transaction);
    }
    if (partialSignerList) {
      // signer has scret key and public key
      for (let i = 0; i < partialSignerList.length; i++) {
        let signer = partialSignerList[i];
        transaction.partialSign(signer);
      }
    }
  } catch (error) {
    return { code: -1, msg: "sign canceled", error };
  }
  // send transaction
  let res = await sendTransaction(connection, signed.serialize());
  if (res.code == 1) {
    return res;
  } else if (res.code == 0) {
    return res;
  } else {
    return { code: 0, msg: "unkown error" };
  }
}

// send transaction and wait for finalize
export async function sendTransaction(connection, transaction) {
  try {
    let signatrue = await connection.sendRawTransaction(transaction);
    let transactionStatus = await getSignatureStatus(connection, signatrue);
    if (transactionStatus) {
      return { code: 1, msg: "send transaction ok", data: signatrue };
    } else {
      return {
        code: 1,
        msg: "can not confirm transaction status please check on explorer",
        data: signatrue,
      };
    }
  } catch (error) {
    console.error("send transaction error", error);
    return { code: 0, msg: "send transaction error", data: error };
  }
}

// check if transaction is finalize
export async function getSignatureStatus(connection, signatrue) {
  let temp = { value: null };
  let flag = true;
  // let startDate = new Date();
  while (flag) {
    temp = await connection.getSignatureStatus(signatrue);
    if (temp.value) {
      // let nowDate = new Date();
      // let passTime = nowDate.getTime() - startDate.getTime();
      // console.log("transaction", temp.value.confirmationStatus, passTime, "ms");
      if (temp.value.confirmationStatus == "finalized") {
        console.log("transaction finalized", signatrue);
        flag = false;
        return temp.value;
      } else {
        await wait(1000);
      }
    }
  }
}

function wait(ms) {
  return new Promise((resolve) => setTimeout(() => resolve(), ms));
}
