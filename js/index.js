import { AccountLayout, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { AmmInstruction } from "./instruction.js";
import { signAndSendTransaction } from "./lib/sendTransction.js";
import {
  getMintData,
  getTokenAccountData,
  getTokenAccountMaxAmount,
} from "./lib/tokenAccount.js";
import { getPoolData, getPoolDataRaw, PoolDataLayout } from "./state.js";

// program
export const AmmProgramId = "aAmLZ9yP1adeZyRC9qMskX9e1Ma2gR4ktpyrDCWPkdm";
const programId = new PublicKey(AmmProgramId);

// token
const USDCKey = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

const PercenMul = 10 ** 6;
const SeedPre = "AMM";
export const Direction = { A2B: 1, B2A: 2 };

export async function createPool(
  connection,
  wallet,
  feeParams,
  amountA,
  amountB,
  tolerance,
  mintAKey,
  mintBKey
) {
  // use account
  let walletAcc = wallet.publicKey;
  // create
  let seed = SeedPre + new Date().getTime().toString();
  let poolAcc = await PublicKey.createWithSeed(walletAcc, seed, programId);
  // check if exist
  let poolData = await connection.getAccountInfo(poolAcc);
  if (poolData) {
    return { code: -2, msg: "pool exist", data: poolAcc.toBase58() };
  }
  let [poolPDA, nonce] = await PublicKey.findProgramAddress(
    [poolAcc.toBuffer()],
    programId
  );
  let mintAAcc = new PublicKey(mintAKey);
  let mintBAcc = new PublicKey(mintBKey);
  let userTokenAKey;
  {
    let res = await getTokenAccountMaxAmount(connection, wallet, mintAKey);
    if (res.code == 1) {
      userTokenAKey = res.data.publicKey;
    } else {
      return res;
    }
  }
  let userTokenBKey;
  {
    let res = await getTokenAccountMaxAmount(connection, wallet, mintBKey);
    if (res.code == 1) {
      userTokenBKey = res.data.publicKey;
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
  let lamportsP = await connection.getMinimumBalanceForRentExemption(
    PoolDataLayout.span
  );
  let lamports = await connection.getMinimumBalanceForRentExemption(
    AccountLayout.span
  );
  let vaultAAccount = new Keypair();
  let vaultBAccount = new Keypair();
  let feeVaultAccount = new Keypair();
  // make transaction
  let tx = new Transaction().add(
    SystemProgram.createAccountWithSeed({
      fromPubkey: walletAcc,
      basePubkey: walletAcc,
      newAccountPubkey: poolAcc,
      seed,
      lamports: lamportsP,
      space: PoolDataLayout.span,
      programId,
    }),
    SystemProgram.createAccount({
      fromPubkey: walletAcc,
      newAccountPubkey: vaultAAccount.publicKey,
      lamports,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    }),
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      mintAAcc,
      vaultAAccount.publicKey,
      poolPDA
    ),
    SystemProgram.createAccount({
      fromPubkey: walletAcc,
      newAccountPubkey: vaultBAccount.publicKey,
      lamports,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    }),
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      mintBAcc,
      vaultBAccount.publicKey,
      poolPDA
    ),
    SystemProgram.createAccount({
      fromPubkey: walletAcc,
      newAccountPubkey: feeVaultAccount.publicKey,
      lamports,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    }),
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      mintBAcc,
      feeVaultAccount.publicKey,
      poolPDA
    ),
    AmmInstruction.createInitInstruction(
      nonce,
      feeParams.rate * PercenMul,
      amountA * 10 ** mintAData.decimals,
      amountB * 10 ** mintBData.decimals,
      tolerance,
      poolAcc,
      walletAcc,
      mintAAcc,
      mintBAcc,
      vaultAAccount.publicKey,
      vaultBAccount.publicKey,
      feeVaultAccount.publicKey,
      poolPDA,
      new PublicKey(userTokenAKey),
      new PublicKey(userTokenBKey),
      TOKEN_PROGRAM_ID,
      programId
    )
  );
  let res = await signAndSendTransaction(
    connection,
    wallet,
    [vaultAAccount, vaultBAccount, feeVaultAccount],
    tx
  );
  if (res.code == 1) {
    return {
      code: 1,
      msg: "init pool ok",
      data: poolAcc.toBase58(),
      signature: res.data,
    };
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
  let poolPDA = await PublicKey.createProgramAddress(
    [poolAcc.toBuffer(), Buffer.from([poolData.nonce])],
    programId
  );
  return { code: 1, msg: "get pda ok", data: poolPDA };
}

export async function findPool(connection) {
  let config = {
    commitment: "finalized",
    filters: [{ dataSize: PoolDataLayout.span }],
  };
  let list = await connection.getParsedProgramAccounts(programId, config);
  return list;
}

export async function findPoolByOwner(connection, ownerKey) {
  let config = {
    commitment: "finalized",
    filters: [
      { memcmp: { offset: 1 * 2 + 8 * 4, bytes: ownerKey } },
      { dataSize: PoolDataLayout.span },
    ],
  };
  let list = await connection.getParsedProgramAccounts(programId, config);
  return list;
}

export async function findPoolByMints(connection, mintAKey, mintBKey) {
  let config = {
    commitment: "finalized",
    filters: [
      { memcmp: { offset: 1 * 2 + 8 * 4 + 32, bytes: mintAKey } },
      { memcmp: { offset: 1 * 2 + 8 * 4 + 32 * 2, bytes: mintBKey } },
      { dataSize: PoolDataLayout.span },
    ],
  };
  let list = await connection.getParsedProgramAccounts(programId, config);
  return list;
}

export function getPoolsData(poolList) {
  return poolList.map((e) => {
    return getPoolDataRaw(e);
  });
}

export async function updateStatus(connection, wallet, poolKey, status) {
  // use account
  let walletAcc = wallet.publicKey;
  let poolAcc = new PublicKey(poolKey);
  // make transaction
  let tx = new Transaction().add(
    AmmInstruction.createUpdateStatusInstrucion(
      status,
      poolAcc,
      walletAcc,
      programId
    )
  );
  let res = await signAndSendTransaction(connection, wallet, null, tx);
  if (res.code == 1) {
    return {
      code: 1,
      msg: "update status ok",
      data: poolAcc.toBase58(),
      signature: res.data,
    };
  } else {
    return res;
  }
}

export async function updateTolerance(connection, wallet, poolKey, tolerance) {
  // use account
  let walletAcc = wallet.publicKey;
  let poolAcc = new PublicKey(poolKey);
  // make transaction
  let tx = new Transaction().add(
    AmmInstruction.createUpdateToleranceInstruction(
      tolerance,
      poolAcc,
      walletAcc,
      programId
    )
  );
  let res = await signAndSendTransaction(connection, wallet, null, tx);
  if (res.code == 1) {
    return {
      code: 1,
      msg: "update tolerance ok",
      data: poolAcc.toBase58(),
      signature: res.data,
    };
  } else {
    return res;
  }
}

export async function terminate(connection, wallet, poolKey) {
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
  {
    let res = await getTokenAccountMaxAmount(
      connection,
      wallet,
      poolData.mint_a
    );
    if (res.code == 1) {
      userTokenAKey = res.data.publicKey;
    } else {
      return res;
    }
  }
  let userTokenBKey;
  {
    let res = await getTokenAccountMaxAmount(
      connection,
      wallet,
      poolData.mint_b
    );
    if (res.code == 1) {
      userTokenBKey = res.data.publicKey;
    } else {
      return res;
    }
  }
  // make transaction
  let tx = new Transaction().add(
    AmmInstruction.createTerminateInstruction(
      poolAcc,
      walletAcc,
      new PublicKey(poolData.vault_a),
      new PublicKey(poolData.vault_b),
      new PublicKey(poolData.fee_vault),
      poolPDA,
      new PublicKey(userTokenAKey),
      new PublicKey(userTokenBKey),
      TOKEN_PROGRAM_ID,
      programId
    )
  );
  let res = await signAndSendTransaction(connection, wallet, null, tx);
  if (res.code == 1) {
    return {
      code: 1,
      msg: "update tolerance ok",
      data: poolAcc.toBase58(),
      signature: res.data,
    };
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
  {
    let res = await getTokenAccountMaxAmount(
      connection,
      wallet,
      poolData.mint_a
    );
    if (res.code == 1) {
      userTokenAKey = res.data.publicKey;
    } else {
      return res;
    }
  }
  let userTokenBKey;
  {
    let res = await getTokenAccountMaxAmount(
      connection,
      wallet,
      poolData.mint_b
    );
    if (res.code == 1) {
      userTokenBKey = res.data.publicKey;
    } else {
      return res;
    }
  }
  // make transaction
  let tx = new Transaction().add(
    AmmInstruction.createSwapInstrucion(
      amount * 10 ** mintAData.decimals,
      direction,
      poolAcc,
      new PublicKey(poolData.vault_a),
      new PublicKey(poolData.vault_b),
      new PublicKey(poolData.fee_vault),
      poolPDA,
      walletAcc,
      new PublicKey(userTokenAKey),
      new PublicKey(userTokenBKey),
      TOKEN_PROGRAM_ID,
      programId
    )
  );
  let res = await signAndSendTransaction(connection, wallet, null, tx);
  if (res.code == 1) {
    return {
      code: 1,
      msg: "swap ok",
      data: "",
      signature: res.data,
    };
  } else {
    return res;
  }
}

export async function getSuperSwapPools(connection, mintAKey, mintBKey) {
  let pool1;
  {
    let list = await findPoolByMints(connection, mintAKey, USDCKey);
    if (list.length > 0) {
      pool1 = list[0].pubkey.toBase58();
    } else {
      return { code: 0, msg: `null pool ${mintAKey}` };
    }
  }
  let pool2;
  {
    let list = await findPoolByMints(connection, mintBKey, USDCKey);
    if (list.length > 0) {
      pool2 = list[0].pubkey.toBase58();
    } else {
      return { code: 0, msg: `null pool ${mintBKey}` };
    }
  }
  return { code: 1, msg: "get supper swap pool ok", data: { pool1, pool2 } };
}

export async function superSwap(
  connection,
  wallet,
  poolKey1,
  poolKey2,
  amount
) {
  // use account
  let walletAcc = wallet.publicKey;
  let poolAcc1 = new PublicKey(poolKey1);
  let poolAcc2 = new PublicKey(poolKey2);
  // use data
  let poolData1;
  {
    let res = await getPoolData(connection, poolKey1);
    if (res.code == 1) {
      poolData1 = res.data;
    } else {
      return res;
    }
  }
  let poolData2;
  {
    let res = await getPoolData(connection, poolKey2);
    if (res.code == 1) {
      poolData2 = res.data;
    } else {
      return res;
    }
  }
  let mintAData;
  {
    let res = await getMintData(connection, poolData1.mint_a);
    if (res.code == 1) {
      mintAData = res.data;
    } else {
      return res;
    }
  }
  let mintBData;
  {
    let res = await getMintData(connection, poolData2.mint_a);
    if (res.code == 1) {
      mintBData = res.data;
    } else {
      return res;
    }
  }
  // use account
  let poolPDA1;
  {
    let res = await getPoolPDA(connection, poolKey1);
    if (res.code == 1) {
      poolPDA1 = res.data;
    } else {
      return res;
    }
  }
  let poolPDA2;
  {
    let res = await getPoolPDA(connection, poolKey2);
    if (res.code == 1) {
      poolPDA2 = res.data;
    } else {
      return res;
    }
  }
  let userTokenAKey;
  {
    let res = await getTokenAccountMaxAmount(
      connection,
      wallet,
      poolData1.mint_a
    );
    if (res.code == 1) {
      userTokenAKey = res.data.publicKey;
    } else {
      return res;
    }
  }
  let userTokenBKey;
  {
    let res = await getTokenAccountMaxAmount(
      connection,
      wallet,
      poolData2.mint_a
    );
    if (res.code == 1) {
      userTokenBKey = res.data.publicKey;
    } else {
      return res;
    }
  }
  let userTokenUSDCKey;
  {
    let res = await getTokenAccountMaxAmount(connection, wallet, USDCKey);
    if (res.code == 1) {
      userTokenUSDCKey = res.data.publicKey;
    } else {
      return res;
    }
  }
  // calculate amount
  let amountB = 0.0;
  {
    // calculate mint a to usdc
    let res = await calculateSwapAmount(
      connection,
      poolKey1,
      amount,
      Direction.A2B
    );
    if (res.code == 1) {
      amountB = res.data;
    } else {
      return res;
    }
  }
  {
    // calculate usdc to mint b
    let res = await calculateSwapAmount2(
      connection,
      poolKey2,
      amountB,
      Direction.B2A
    );
    if (res.code == 1) {
      amountB = res.data;
    } else {
      return res;
    }
  }
  // make transaction
  let tx = new Transaction().add(
    AmmInstruction.createSwapInstrucion(
      amount * 10 ** mintAData.decimals,
      Direction.A2B,
      poolAcc1,
      new PublicKey(poolData1.vault_a),
      new PublicKey(poolData1.vault_b),
      new PublicKey(poolData1.fee_vault),
      poolPDA1,
      walletAcc,
      new PublicKey(userTokenAKey),
      new PublicKey(userTokenUSDCKey),
      TOKEN_PROGRAM_ID,
      programId
    ),
    AmmInstruction.createSwapInstrucion(
      amountB * 10 ** mintBData.decimals,
      Direction.B2A,
      poolAcc2,
      new PublicKey(poolData2.vault_a),
      new PublicKey(poolData2.vault_b),
      new PublicKey(poolData2.fee_vault),
      poolPDA2,
      walletAcc,
      new PublicKey(userTokenBKey),
      new PublicKey(userTokenUSDCKey),
      TOKEN_PROGRAM_ID,
      programId
    )
  );
  let res = await signAndSendTransaction(connection, wallet, null, tx);
  if (res.code == 1) {
    return {
      code: 1,
      msg: "super swap ok",
      data: "",
      signature: res.data,
    };
  } else {
    return res;
  }
}

export async function withdrawalFee(connection, wallet, poolKey) {
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
  // use account
  let userTokenBKey;
  {
    let res = await getTokenAccountMaxAmount(
      connection,
      wallet,
      poolData.mint_b
    );
    if (res.code == 1) {
      userTokenBKey = res.data.publicKey;
    } else {
      return res;
    }
  }
  let poolPDA;
  {
    let res = await getPoolPDA(connection, poolKey);
    if (res.code == 1) {
      poolPDA = res.data;
    } else {
      return res;
    }
  }
  // make transaction
  let tx = new Transaction().add(
    AmmInstruction.createWithdrawalFeeInstruction(
      poolAcc,
      walletAcc,
      new PublicKey(poolData.fee_vault),
      new PublicKey(userTokenBKey),
      poolPDA,
      TOKEN_PROGRAM_ID,
      programId
    )
  );
  let res = await signAndSendTransaction(connection, wallet, null, tx);
  if (res.code == 1) {
    return {
      code: 1,
      msg: "update status ok",
      data: poolAcc.toBase58(),
      signature: res.data,
    };
  } else {
    return res;
  }
}

export async function calculateSwapAmount(
  connection,
  poolKey,
  amount,
  direction
) {
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
  let vaultAData;
  {
    let res = await getTokenAccountData(connection, poolData.vault_a);
    if (res.code == 1) {
      vaultAData = res.data;
    } else {
      return res;
    }
  }
  let vaultBData;
  {
    let res = await getTokenAccountData(connection, poolData.vault_b);
    if (res.code == 1) {
      vaultBData = res.data;
    } else {
      return res;
    }
  }
  // calculate
  let k = pool.ka * pool.kb;
  let A = vaultAData.amount * 10 ** vaultAData.decimals;
  let B = vaultBData.amount * 10 ** vaultBData.decimals;
  let a = amount;
  let b = 0;
  let kNew = 0;
  if ((direction = Direction.A2B)) {
    b = Math.round(B - k / (A + a));
    if (b >= B) {
      return { code: -1, msg: "b is greater than B", data: b };
    }
    kNew = (A + a) * (B - b);
  } else if ((direction = Direction.B2A)) {
    if (a >= A) {
      return { code: -2, msg: "a is greater than A", data: a };
    }
    b = Math.round(k / (A - a) - B);
    kNew = (A - a) * (B + b);
  } else {
    return { code: -3, msg: "direction unknow", data: direction };
  }
  // check tolerance
  // let diff = Math.abs(k - kNew);
  // if (diff > pool.tolerance) {
  //   return { code: -4, msg: "out of tolerance", data: diff };
  // }
  return { code: 1, msg: "calculate swap amount ok", data: b };
}

export async function calculateSwapAmount2(
  connection,
  poolKey,
  amount,
  direction
) {
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
  let vaultAData;
  {
    let res = await getTokenAccountData(connection, poolData.vault_a);
    if (res.code == 1) {
      vaultAData = res.data;
    } else {
      return res;
    }
  }
  let vaultBData;
  {
    let res = await getTokenAccountData(connection, poolData.vault_b);
    if (res.code == 1) {
      vaultBData = res.data;
    } else {
      return res;
    }
  }
  // calculate
  let k = pool.ka * pool.kb;
  let A = vaultAData.amount * 10 ** vaultAData.decimals;
  let B = vaultBData.amount * 10 ** vaultBData.decimals;
  let a = 0;
  let b = amount;
  let kNew = 0;
  if ((direction = Direction.A2B)) {
    a = Math.round(k / (B - b) - A);
    if (a >= A) {
      return { code: -2, msg: "a is greater than A", data: a };
    }
    kNew = (A + a) * (B - b);
  } else if ((direction = Direction.B2A)) {
    if (b >= B) {
      return { code: -1, msg: "b is greater than B", data: b };
    }
    b = Math.round(A - k / (B + b));
    kNew = (A - a) * (B + b);
  } else {
    return { code: -3, msg: "direction unknow", data: direction };
  }
  // check tolerance
  // let diff = Math.abs(k - kNew);
  // if (diff > pool.tolerance) {
  //   return { code: -4, msg: "out of tolerance", data: diff };
  // }
  return { code: 1, msg: "calculate swap amount ok", data: b };
}

export { getPoolData };
