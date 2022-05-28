# Automated market makers

## Overview

Automated market makers smart contract

**This code has NOT been audit, use on your own risk.**

## Software framework

read https://docs.solana.com/developing/programming-model/overview

## Design

### Role

1. Owner
2. User

### Business

1. Create

   Owner creats amm pool, transfer two kinds of tokens when creating.

   Then we have the fixed k, k value is the product of tokens' amount.

2. Swap

   User transfers token into amm pool, for swapping another token.

   Amounts of transfer is follow the formula below.
   $$
   (a-∆a)*(b+∆b)=k
   $$
   In fact, because of calculate resolution is limited, there will be a little error.

3. Terminate

   Owner terminate the amm pool, withdrawal all tokens, and close all account.

4. fee

   User transfers some extra token as fee, when swapping.

   The fee mint and rate is configured by owner, when creating.

   Owner could withdrawal fee any time.

   Owner will withdrawal all fee when terminating.


## Install

1. install rustc v1.56.1, read https://www.rust-lang.org/tools/install

2. install solana cli v1.9.5, read https://docs.solana.com/cli/install-solana-cli-tools

3. confirm id.json file path

4. build

   ```bash
   cargo build-bpf
   ```

5. confirm id.json file wallet has enough SOL.

6. deploy

   ```bash
   solana program deploy target/deploy/amm.so
   ```

7. edit AmmProgramId which is in js/index.js 

8. test

   ```bash
   npm run test
   ```

## File

1. src/ smart contract code
2. js/ js code for calling smart contract
3. client/ client application

## Known Problem

1. When token amount is zero, or it would be zero, swap will fail.
2. Fee is calculated base on token amount, it might be error because of decimals.
3. There isn't a reasonable for checking swap calculation error. 

## Plan

super swap

There is two amm pool, they has token a/b and b/c.

Then it could swap twice, swap a/c by using b as middle template.

## Useful commandd

```bash
export PATH="/home/ubuntu/.local/share/solana/install/active_release/bin:$PATH"
solana program deploy amm.so
```

