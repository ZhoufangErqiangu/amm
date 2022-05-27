import {
  MintLayout,
  AccountLayout,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
} from '@solana/spl-token';
import { Keypair, PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import { signAndSendTransaction } from './sendTransction.js';

export async function createTokenAccount(connection, wallet, mintKey) {
  // use account
  let walletAcc = wallet.publicKey;
  // create account
  let newAccount = new Keypair();
  let mintAcc = new PublicKey(mintKey);
  let lamports = await connection.getMinimumBalanceForRentExemption(AccountLayout.span);
  // make transction
  let tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: walletAcc,
      newAccountPubkey: newAccount.publicKey,
      lamports,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    }),
    Token.createInitAccountInstruction(TOKEN_PROGRAM_ID, mintAcc, newAccount.publicKey, walletAcc)
  );
  let res = await signAndSendTransaction(connection, wallet, [newAccount], tx);
  if (res.code == 1) {
    return {
      code: 1,
      msg: 'token account create ok',
      data: newAccount.publicKey.toBase58(),
      signature: res.data,
    };
  } else {
    return res;
  }
}

export async function createAssociatedTokenAccount(connection, wallet, mintKey) {
  // use account
  let walletAcc = wallet.publicKey;
  // create account
  let newAccount = new Keypair();
  let mintAcc = new PublicKey(mintKey);
  let lamports = await connection.getMinimumBalanceForRentExemption(AccountLayout.span);
  // make transction
  let tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: walletAcc,
      newAccountPubkey: newAccount.publicKey,
      lamports,
      space: AccountLayout.span,
      programId: ASSOCIATED_TOKEN_PROGRAM_ID,
    }),
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mintAcc,
      newAccount.publicKey,
      walletAcc,
      walletAcc
    )
  );
  let res = await signAndSendTransaction(connection, wallet, [newAccount], tx);
  if (res.code == 1) {
    return {
      code: 1,
      msg: 'token account create ok',
      data: newAccount.publicKey.toBase58(),
      signature: res.data,
    };
  } else {
    return res;
  }
}

export async function getTokenAccountData(connection, tokenKey) {
  let tokenAcc = new PublicKey(tokenKey);
  let temp = await connection.getParsedAccountInfo(tokenAcc);
  if (temp.value) {
    let info = temp.value.data.parsed.info;
    let amount = info.tokenAmount.uiAmount;
    let decimals = info.tokenAmount.decimals;
    return {
      code: 1,
      msg: 'get token account data ok',
      data: {
        publicKey: tokenKey,
        owner: info.owner,
        mint: info.mint,
        amount,
        decimals,
      },
    };
  } else {
    return { code: 0, msg: 'account is null', data: null };
  }
}

export async function getTokenAccountMaxAmount(connection, wallet, mintKey) {
  // use account
  let walletAcc = wallet.publicKey;
  let mintAcc = new PublicKey(mintKey);
  // find token accounts
  let res = await connection.getTokenAccountsByOwner(walletAcc, {
    mint: mintAcc,
  });
  if (res.value.length == 1) {
    // user has only one token aacount
    let res1 = await getTokenAccountData(connection, res.value[0].pubkey.toBase58());
    if (res1.code == 1) {
      return {
        code: 1,
        msg: 'user has only one token account',
        data: res1.data,
      };
    } else {
      return res1;
    }
  } else if (res.value.length == 0) {
    // user has no token account
    return { code: -1, msg: 'user has no token account', data: null };
  } else {
    // get token account amount
    let accounts = [];
    for (let i = 0; i < res.value.length; i++) {
      let tempAcc = res.value[i].pubkey;
      let res1 = await getTokenAccountData(connection, tempAcc.toBase58());
      if (res1.code == 1) {
        accounts.push(res1.data);
      }
    }
    // find which token account has max amount
    if (accounts.length > 0) {
      let amounts = accounts.map((e) => {
        return e.amount;
      });
      let maxAmount = Math.max(...amounts);
      let maxIndex = amounts.indexOf(maxAmount);
      if (maxIndex != -1) {
        return {
          code: 1,
          msg: 'find token account ok',
          data: accounts[maxIndex],
        };
      } else {
        return {
          code: 0,
          msg: 'can not find amount max token account',
          data: null,
        };
      }
    } else {
      return {
        code: 0,
        msg: 'can not find amount max token account',
        data: null,
      };
    }
  }
}

export async function createMintAccount(connection, wallet, decimals = 9) {
  // use account
  let walletAcc = wallet.publicKey;
  let lamports = await connection.getMinimumBalanceForRentExemption(MintLayout.span);
  let mintAccount = new Keypair();
  let tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: walletAcc,
      newAccountPubkey: mintAccount.publicKey,
      lamports,
      space: MintLayout.span,
      programId: TOKEN_PROGRAM_ID,
    }),
    Token.createInitMintInstruction(
      TOKEN_PROGRAM_ID,
      mintAccount.publicKey,
      decimals,
      walletAcc,
      null
    )
  );
  let res = await signAndSendTransaction(connection, wallet, [mintAccount], tx);
  if (res.code == 1) {
    return {
      code: 1,
      msg: 'token account create ok',
      data: mintAccount.publicKey.toBase58(),
      signature: res.data,
    };
  } else {
    return res;
  }
}

export async function getMintData(connection, mintKey) {
  // use account
  let mintAcc = new PublicKey(mintKey);
  let temp = await connection.getParsedAccountInfo(mintAcc);
  if (temp.value) {
    let info = temp.value.data.parsed.info;
    let supply = parseFloat(info.supply);
    let decimals = info.decimals;
    return {
      code: 1,
      msg: 'get mint data ok',
      data: {
        publicKey: mintKey,
        mintAuthority: info.mintAuthority,
        freezeAuthority: info.freezeAuthority,
        supply,
        decimals,
      },
    };
  } else {
    return { code: 0, msg: 'account is null', data: null };
  }
}

export async function mintToTokenAccount(connection, wallet, userTokenKey, amount) {
  // use account
  let walletAcc = wallet.publicKey;
  let userTokenAcc = new PublicKey(userTokenKey);
  // use data
  let userTokenData;
  {
    let res = await getTokenAccountData(connection, userTokenKey);
    if (res.code == 1) {
      userTokenData = res.data;
    } else {
      return res;
    }
  }
  // make transaction
  let tx = new Transaction().add(
    Token.createMintToInstruction(
      TOKEN_PROGRAM_ID,
      new PublicKey(userTokenData.mint),
      userTokenAcc,
      walletAcc,
      [],
      amount * 10 ** userTokenData.decimals
    )
  );
  let res = await signAndSendTransaction(connection, wallet, null, tx);
  if (res.code == 1) {
    return { code: 1, msg: 'mint to ok', signature: res.data };
  } else {
    return res;
  }
}
