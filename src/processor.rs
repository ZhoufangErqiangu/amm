//! Program state processor

use solana_program::sysvar::Sysvar;
use {
    crate::{error::AmmError, instruction::SapInstruction, state::AmmPool},
    arrayref::{array_ref, array_refs},
    // std::convert::TryInto,
    chainlink_solana,
    num_traits::FromPrimitive,
    // pyth_client::{CorpAction, PriceStatus, PriceType},
    solana_program::{
        account_info::AccountInfo,
        clock::Clock,
        decode_error::DecodeError,
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::{PrintProgramError, ProgramError},
        program_option::COption,
        program_pack::Pack,
        pubkey::Pubkey,
        // sysvar::Sysvar,
        // commitment_config::CommitmentConfig,
    },
};

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Processes [Instruction](enum.Instruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = SapInstruction::unpack(input)?;
        match instruction {
            SapInstruction::Initialize {
                nonce,
                fee_1,
                fee_2,
                fee_3,
                fee_4,
                fee_5,
            } => {
                msg!("Instruction: Init");
                Self::process_initialize(
                    program_id, accounts, nonce, fee_1, fee_2, fee_3, fee_4, fee_5,
                )
            }
            SapInstruction::UpdatePool {} => {
                msg!("Instruction: Update Pool");
                Self::process_update_pool(program_id, accounts)
            }
            SapInstruction::UpdateStatus { status } => {
                msg!("Instruction: Update Status");
                Self::process_update_status(program_id, accounts, status)
            }
            SapInstruction::Swap { amount } => {
                msg!("Instruction: Swap");
                Self::process_swap(program_id, accounts, amount)
            }
        }
    }

    /// Unpacks a spl_token `Account`.
    fn unpack_token_account(
        account_info: &AccountInfo,
        token_program_id: &Pubkey,
    ) -> Result<spl_token::state::Account, AmmError> {
        if account_info.owner != token_program_id {
            Err(AmmError::InvalidTokenProgramId)
        } else {
            spl_token::state::Account::unpack(&account_info.data.borrow())
                .map_err(|_| AmmError::ExpectedAccount)
        }
    }

    /// Unpacks a spl_token `Mint`.
    fn unpack_mint(
        account_info: &AccountInfo,
        token_program_id: &Pubkey,
    ) -> Result<spl_token::state::Mint, AmmError> {
        if account_info.owner != token_program_id {
            Err(AmmError::InvalidTokenProgramId)
        } else {
            spl_token::state::Mint::unpack(&account_info.data.borrow())
                .map_err(|_| AmmError::ExpectedMint)
        }
    }

    /// Calculates the authority id by generating a program address.
    fn authority_id(program_id: &Pubkey, my_info: &Pubkey, nonce: u8) -> Result<Pubkey, AmmError> {
        msg!("program id:{:?}", program_id);
        msg!("my info:{:?}, nonce:{:?}", my_info, nonce);
        let ak = Pubkey::create_program_address(&[&my_info.to_bytes()[..32], &[nonce]], program_id);
        msg!("pda:{:?}", ak);
        Err(AmmError::InvalidProgramAddress)
        // .or(Err(AmmError::InvalidProgramAddress))
    }

    /// Issue a spl_token `Burn` instruction.
    fn token_burn<'a>(
        _sap_pool: &Pubkey,
        token_program: AccountInfo<'a>,
        burn_account: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        _nonce: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let ix = spl_token::instruction::burn(
            token_program.key,
            burn_account.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke(&ix, &[burn_account, mint, authority, token_program])
    }

    /// Issue a spl_token `MintTo` instruction.
    fn token_mint_to<'a>(
        _sap_pool: &Pubkey,
        token_program: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        _nonce: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let authority_bytes = authority.key.to_bytes();
        let authority_signature_seeds = [&authority_bytes[..32]];
        let signers = &[&authority_signature_seeds[..]];
        let ix = spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            authority.key,
            &[&authority.key],
            amount,
        )?;
        invoke_signed(&ix, &[mint, destination, authority, token_program], signers)
    }

    /// Issue a spl_token `MintTo` instruction.
    fn token_mint_to2<'a>(
        sap_pool: &Pubkey,
        token_program: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        pda: AccountInfo<'a>,
        nonce: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let seeds = &[sap_pool.as_ref(), &[nonce]];
        let signer = &[&seeds[..]];
        let ix = spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            pda.key,
            &[],
            amount,
        )?;
        invoke_signed(&ix, &[mint, destination, pda, token_program], signer)
    }

    /// Issue a spl_token `Transfer` instruction.
    fn token_transfer<'a>(
        _sap_pool: &Pubkey,
        token_program: AccountInfo<'a>,
        source: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        _nonce: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let ix = spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[&authority.key],
            amount,
        )?;
        invoke(
            &ix,
            &[source, destination, authority, token_program], // signers,
        )
    }

    /// Processes `Initialize` instruction.
    fn process_initialize(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        nonce: u8,
        fee_1: f64,
        fee_2: f64,
        fee_3: f64,
        fee_4: f64,
        fee_5: f64,
    ) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 12];
        let [pool_acc, owner_acc, mint_a_acc, mint_b_acc, vault_a_acc, vault_b_acc, fee_receiver_1_acc, fee_receiver_2_acc, fee_receiver_3_acc, fee_receiver_4_acc, fee_receiver_5_acc, pool_pda] =
            accounts;

        // use data
        let mut pool = AmmPool::unpack(&pool_acc.data.borrow())?;
        // check
        if !owner_acc.is_signer {
            msg!("owner must sign");
            return Err(AmmError::InvalidSignAccount.into());
        }
        if pool.owner != NULL_PUBKEY {
            return Err(AmmError::AmmPoolExist.into());
        }
        let vault_a = Self::unpack_token_account(vault_a_acc, &spl_token::ID)?;
        if vault_a.mint != *mint_a_acc.key {
            msg!(
                "vault a mint not match {} {}",
                vault_a.mint,
                *mint_a_acc.key
            );
            return Err(AmmError::InvalidMint.into());
        }
        let vault_b = Self::unpack_token_account(vault_b_acc, &spl_token::ID)?;
        if vault_b.mint != *mint_b_acc.key {
            msg!(
                "vault b mint not match {} {}",
                vault_b.mint,
                *mint_b_acc.key
            );
            return Err(AmmError::InvalidMint.into());
        }
        // init pool
        pool.status = 1;
        pool.nonce = nonce;
        pool.fee_1 = fee_1;
        pool.fee_2 = fee_2;
        pool.fee_3 = fee_3;
        pool.fee_4 = fee_4;
        pool.fee_5 = fee_5;
        pool.owner = *owner_acc.key;
        pool.mint_a = *owner_acc.key;
        pool.mint_b = *owner_acc.key;
        pool.vault_a = *owner_acc.key;
        pool.vault_b = *owner_acc.key;
        pool.fee_receiver_1 = *fee_receiver_1_acc.key;
        pool.fee_receiver_2 = *fee_receiver_2_acc.key;
        pool.fee_receiver_3 = *fee_receiver_3_acc.key;
        pool.fee_receiver_4 = *fee_receiver_4_acc.key;
        pool.fee_receiver_5 = *fee_receiver_5_acc.key;

        AmmPool::pack(pool, &mut pool_acc.data.borrow_mut())?;
        Ok(())
    }

    /// Processes `Update Pool` instruction.
    fn process_update_pool(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 10];
        let [owner_acc, token_program_acc, sap_pool_acc, sap_pool_l1_mint_acc, sap_pool_l2_mint_acc, user_sap_acc, user_wallet_acc, user_token_acc, clock_acc, _chainlink_program_acc] =
            accounts;
        if !user_wallet_acc.is_signer {
            return Err(AmmError::InvalidSignAccount.into());
        }
        let mut this_sap_pool = AmmPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        let clock = Clock::from_account_info(&clock_acc)?;
        // check
        // 3 is free mint burn
        if this_sap_pool.status != 3 {
            return Err(AmmError::AmmPoolLock.into());
        }
        if this_sap_pool.sap_mint_l1_pubkey != *sap_pool_l1_mint_acc.key {
            return Err(AmmError::InvalidMint.into());
        }
        if this_sap_pool.sap_mint_l2_pubkey != *sap_pool_l2_mint_acc.key {
            return Err(AmmError::InvalidMint.into());
        }
        if *token_program_acc.key != spl_token::ID {
            return Err(AmmError::InvalidTokenProgramId.into());
        }
        let token_program_id = *token_program_acc.key;
        let user_sap_account = Self::unpack_token_account(user_sap_acc, &token_program_id)?;
        let user_token_account = Self::unpack_token_account(user_token_acc, &token_program_id)?;
        let sap_pool_l1_mint_account = Self::unpack_mint(sap_pool_l1_mint_acc, &token_program_id)?;
        let sap_pool_l2_mint_account = Self::unpack_mint(sap_pool_l2_mint_acc, &token_program_id)?;
        let sap_pool_mint_acc;
        let sap_pool_mint_account;
        if ltype == 1 {
            sap_pool_mint_account = sap_pool_l1_mint_account.clone();
            sap_pool_mint_acc = sap_pool_l1_mint_acc.clone();
            if user_sap_account.mint != this_sap_pool.sap_mint_l1_pubkey {
                return Err(AmmError::InvalidMint.into());
            }
        } else if ltype == 2 {
            sap_pool_mint_account = sap_pool_l2_mint_account.clone();
            sap_pool_mint_acc = sap_pool_l2_mint_acc.clone();
            if user_sap_account.mint != this_sap_pool.sap_mint_l2_pubkey {
                return Err(AmmError::InvalidMint.into());
            }
        } else {
            return Err(AmmError::InvalidMint.into());
        }
        // let sap_pool_mint_account = Self::unpack_mint(&sap_pool_mint_acc, &token_program_id)?;
        // check account
        if user_sap_account.owner != *user_wallet_acc.key {
            return Err(AmmError::InvalidUserOwner.into());
        }
        if user_token_account.amount < amount_in {
            return Err(AmmError::InsufficientFunds.into());
        }
        if user_token_account.mint != *token_pubkey {
            return Err(AmmError::InvalidMint.into());
        }

        // check user token acc mint belongs to one of the token list
        if !(&this_sap_pool
            .token_list
            .iter()
            .any(|v| v == &user_token_account.mint))
        {
            return Err(AmmError::BaseMintNoMatch.into());
        }
        // step1: Calculates total value of the asset
        // calculate sap price
        let mut total_value: f64 = 0.0;
        // let user_token_value: u64 = 0;
        let mut this_token_vault_acc = user_token_acc.clone();
        let mut this_token_mint_acc = user_token_acc.clone();
        let mut selected_token_price: f64 = 0.0;
        for i in 0..TOKEN_NUM {
            // use account
            let group_accs = array_ref![groups_accs, i * LIST_NUM, LIST_NUM];
            let [sap_token_mint_acc, sap_token_vault_acc, oracle_price_acc] = group_accs;
            // use data
            let sap_token_vault_account =
                Self::unpack_token_account(sap_token_vault_acc, &token_program_id)?;
            let sap_token_mint_account = Self::unpack_mint(sap_token_mint_acc, &token_program_id)?;
            // check account
            if sap_token_vault_acc.key != &this_sap_pool.token_assets_vault_pubkeys[i] {
                return Err(AmmError::InvalidVault.into());
            }
            if &this_sap_pool.oracle_price_pubkey_list[i] != oracle_price_acc.key {
                return Err(AmmError::OraclePriceMissMatch.into());
            }
            let token_result: f64;
            // syp has no oracle, its price is fixed
            if *sap_token_mint_acc.key == SYP_PUBKEY {
                token_result = SYP_PRICE;
            } else {
                // get oracle price
                token_result = get_token_price_only(oracle_price_acc)?;
            }
            // msg!("token pricd:{}, total value:{}", token_result, total_value);
            total_value += sap_token_vault_account.amount as f64 * token_result
                / (u64::pow(10, sap_token_mint_account.decimals as u32) as f64);
            if sap_token_mint_acc.key == token_pubkey {
                selected_token_price = token_result; //price from oracle
                this_token_vault_acc = sap_token_vault_acc.clone();
                this_token_mint_acc = sap_token_mint_acc.clone();
            }
        }
        // msg!("selected token price:{}", selected_token_price);
        let pre_amount1: f64 = this_sap_pool.sap_pre_mint_l1_amount as f64
            / (u64::pow(10, sap_pool_l1_mint_account.decimals as u32) as f64);
        let supply1: f64 = sap_pool_l1_mint_account.supply as f64
            / (u64::pow(10, sap_pool_l1_mint_account.decimals as u32) as f64)
            + pre_amount1;
        let pre_amount2: f64 = this_sap_pool.sap_pre_mint_l2_amount as f64
            / (u64::pow(10, sap_pool_l2_mint_account.decimals as u32) as f64);
        let supply2: f64 = sap_pool_l2_mint_account.supply as f64
            / (u64::pow(10, sap_pool_l2_mint_account.decimals as u32) as f64)
            + pre_amount2;
        // msg!("total value:{}", total_value);
        // minus fee
        total_value -= this_sap_pool.fee;
        total_value -= this_sap_pool.performance_fee;
        if total_value < 0.0 {
            total_value = 0.0;
        }
        // msg!(
        //     "current fee: {}, performance fee: {}",
        //     this_sap_pool.fee,
        //     this_sap_pool.performance_fee
        // );
        let sap_price: f64 = total_value as f64 / supply1 + supply2;
        // msg!(
        //     "total value: {}, sap supply: {}, sap price: {}",
        //     total_value,
        //     supply1 + supply2,
        //     sap_price
        // );
        // calculate sy price
        let this_token_mint_account = Self::unpack_mint(&this_token_mint_acc, &token_program_id)?;
        let past_time: f64 = (clock.unix_timestamp - this_sap_pool.sap_init_ts) as f64;
        let sy_day = (past_time / DAY_TIME).round();
        // msg!("sy day:{}", sy_day);
        let unit_sap_price: f64;
        let rewards: [f64; 2] = find_rewards_price(
            sap_price,
            supply1,
            supply2,
            this_sap_pool.sap_init_price,
            sy_day,
        );
        if ltype == 1 {
            unit_sap_price = this_sap_pool.sap_init_price * (1.0 + rewards[0]);
            msg!("sy price 1:{}", unit_sap_price);
        } else if ltype == 2 {
            unit_sap_price = this_sap_pool.sap_init_price * (1.0 + rewards[1]);
            msg!(
                "sy price 2:{}, supply 1:{}, supply 2:{}",
                unit_sap_price,
                supply1,
                supply2
            );
        } else {
            return Err(AmmError::InvalidMint.into());
        }
        let mint_fee = fee as f64 / PERCENT_MUL;
        let mint_value: f64 = unit_sap_price * amount_in as f64
            / (u64::pow(10, sap_pool_mint_account.decimals as u32) as f64);
        // msg!(
        //     "mint value:{}, token price:{}",
        //     mint_value,
        //     selected_token_price
        // );
        let mint_fee_value = mint_value * mint_fee;
        // save fee
        this_sap_pool.fee += mint_fee_value;
        // msg!("mint fee:{}", mint_fee_value);
        AmmPool::pack(this_sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        // calculate transfer amount
        let amount_out = (mint_value + mint_fee_value) / selected_token_price;
        let transfer_amount =
            amount_out * (u64::pow(10, this_token_mint_account.decimals as u32) as f64);
        // msg!("transfer amount:{}", transfer_amount);
        // transfer to valut
        Self::token_transfer(
            sap_pool_acc.key,
            token_program_acc.clone(),
            user_token_acc.clone(),
            this_token_vault_acc.clone(),
            user_wallet_acc.clone(),
            this_sap_pool.nonce as u8,
            transfer_amount as u64,
        )?;
        // mint sap token
        Self::token_mint_to(
            sap_pool_acc.key,
            token_program_acc.clone(),
            sap_pool_mint_acc.clone(),
            user_sap_acc.clone(),
            owner_acc.clone(),
            this_sap_pool.nonce as u8,
            amount_in,
        )?;

        Ok(())
    }

    /// Processes `Update Status` instruction.
    fn process_update_status(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        status: u8,
    ) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 10];
        let [token_program_acc, sap_pool_acc, sap_pool_l1_mint_acc, sap_pool_l2_mint_acc, user_sap_acc, user_wallet_acc, user_token_acc, manager_acc, clock_acc, _chainlink_program_acc] =
            accounts;

        // use data
        let mut this_sap_pool = AmmPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        let clock = Clock::from_account_info(&clock_acc)?;
        // check
        // 3 is free mint burn
        if this_sap_pool.status != 3 {
            return Err(AmmError::AmmPoolLock.into());
        }
        if this_sap_pool.sap_mint_l1_pubkey != *sap_pool_l1_mint_acc.key {
            return Err(AmmError::InvalidMint.into());
        }
        if this_sap_pool.sap_mint_l2_pubkey != *sap_pool_l2_mint_acc.key {
            return Err(AmmError::InvalidMint.into());
        }
        let token_program_id = *token_program_acc.key;
        let user_sap_account = Self::unpack_token_account(user_sap_acc, &token_program_id)?;
        let user_token_account = Self::unpack_token_account(user_token_acc, &token_program_id)?;
        let sap_pool_l1_mint_account = Self::unpack_mint(sap_pool_l1_mint_acc, &token_program_id)?;
        let sap_pool_l2_mint_account = Self::unpack_mint(sap_pool_l2_mint_acc, &token_program_id)?;
        let now_time = clock.unix_timestamp;
        let past_time: f64 = (now_time - this_sap_pool.sap_init_ts) as f64;
        let sy_day = (past_time / DAY_TIME).round();
        let sap_pool_mint_acc;
        let sap_pool_mint_account;
        if ltype == 1 {
            sap_pool_mint_account = sap_pool_l1_mint_account.clone();
            sap_pool_mint_acc = sap_pool_l1_mint_acc.clone();
            if user_sap_account.mint != this_sap_pool.sap_mint_l1_pubkey {
                return Err(AmmError::InvalidMint.into());
            }
        } else if ltype == 2 {
            sap_pool_mint_account = sap_pool_l2_mint_account.clone();
            sap_pool_mint_acc = sap_pool_l2_mint_acc.clone();
            if user_sap_account.mint != this_sap_pool.sap_mint_l2_pubkey {
                return Err(AmmError::InvalidMint.into());
            }
        } else {
            return Err(AmmError::InvalidMint.into());
        }

        // check signer
        if !user_wallet_acc.is_signer {
            return Err(AmmError::InvalidSignAccount.into());
        }
        // check account
        if amount_in == 0 {
            return Err(AmmError::InvalidInput.into());
        }
        if *token_program_acc.key != spl_token::ID {
            return Err(AmmError::InvalidTokenProgramId.into());
        }
        if user_sap_account.owner != *user_wallet_acc.key {
            return Err(AmmError::InvalidUserOwner.into());
        }
        if user_token_account.owner != *user_wallet_acc.key {
            return Err(AmmError::InvalidUserOwner.into());
        }
        if user_sap_account.amount < amount_in {
            return Err(AmmError::InsufficientFunds.into());
        }
        // check user token acc mint belongs to one of the token list
        if !(&this_sap_pool
            .token_list
            .iter()
            .any(|v| v == &user_token_account.mint))
        {
            return Err(AmmError::BaseMintNoMatch.into());
        }

        // step1: Calculates total value of the asset
        let mut total_value: f64 = 0.0;
        // let user_token_value: u64 = 0;
        let mut this_token_vault_acc = user_token_acc.clone();
        let mut this_token_mint_acc = user_token_acc.clone();
        let mut selected_token_price: f64 = 1.0;
        for i in 0..TOKEN_NUM {
            // use account
            let group_accs = array_ref![groups_accs, i * LIST_NUM, LIST_NUM];
            let [sap_token_mint_acc, sap_token_vault_acc, oracle_price_acc] = group_accs;
            // use data
            let sap_token_vault_account =
                Self::unpack_token_account(sap_token_vault_acc, &token_program_id)?;
            let sap_token_mint_account = Self::unpack_mint(sap_token_mint_acc, &token_program_id)?;
            // check account
            if sap_token_vault_acc.key != &this_sap_pool.token_assets_vault_pubkeys[i] {
                return Err(AmmError::InvalidVault.into());
            }
            if &this_sap_pool.oracle_price_pubkey_list[i] != oracle_price_acc.key {
                return Err(AmmError::OraclePriceMissMatch.into());
            }
            let token_result: f64;
            // syp has no oracle, its price is fixed
            if *sap_token_mint_acc.key == SYP_PUBKEY {
                token_result = SYP_PRICE;
            } else {
                // get oracle price
                token_result = get_token_price_only(oracle_price_acc)?;
            }
            let token_price = token_result;
            total_value += sap_token_vault_account.amount as f64 * token_price
                / (u64::pow(10, sap_token_mint_account.decimals as u32) as f64);
            if sap_token_mint_acc.key == &user_token_account.mint {
                selected_token_price = token_price;
                this_token_vault_acc = sap_token_vault_acc.clone();
                this_token_mint_acc = sap_token_mint_acc.clone();
            }
        }
        // calculate sap price
        let supply1: f64 = sap_pool_l1_mint_account.supply as f64
            / (u64::pow(10, sap_pool_l1_mint_account.decimals as u32) as f64);
        let supply2: f64 = sap_pool_l2_mint_account.supply as f64
            / (u64::pow(10, sap_pool_l2_mint_account.decimals as u32) as f64);
        let supply3: f64 = supply1 + supply2;
        // msg!("total value:{}", total_value);
        // minus fee
        total_value -= this_sap_pool.fee;
        total_value -= this_sap_pool.performance_fee;
        // msg!(
        //     "fee: {}, performance fee: {}",
        //     this_sap_pool.fee,
        //     this_sap_pool.performance_fee
        // );
        let sap_price: f64 = total_value / supply3;
        // msg!(
        //     "total value: {}, sap supply: {}, sap price: {}",
        //     total_value,
        //     supply3,
        //     sap_price
        // );
        // calculate sy price
        // msg!("sy day:{}", sy_day);
        let rewards: [f64; 2] = find_rewards_price(
            sap_price,
            supply1,
            supply2,
            this_sap_pool.sap_init_price,
            sy_day,
        );
        let unit_sap_price: f64;
        if ltype == 1 {
            unit_sap_price = this_sap_pool.sap_init_price * (1.0 + rewards[0]);
            msg!(
                "sy price 1:{}, supply 1:{}, supply 2:{}",
                unit_sap_price,
                supply1,
                supply2
            );
        } else if ltype == 2 {
            unit_sap_price = this_sap_pool.sap_init_price * (1.0 + rewards[1]);
            msg!(
                "sy price 2:{}, supply 1:{}, supply 2:{}",
                unit_sap_price,
                supply1,
                supply2
            );
        } else {
            return Err(AmmError::InvalidMint.into());
        }
        // calculate value
        let burn_value = unit_sap_price * amount_in as f64
            / (u64::pow(10, sap_pool_mint_account.decimals as u32) as f64);
        // msg!(
        //     "burn value:{}, token price:{}",
        //     burn_value,
        //     selected_token_price
        // );
        // calculate fee
        let sap_cost1 = sap_cost as f64 / PERCENT_MUL;
        let burn_fee_value = burn_fee as f64 / PERCENT_MUL * burn_value;
        let performance_fee_value: f64;
        if unit_sap_price > sap_cost1 {
            performance_fee_value = (unit_sap_price - sap_cost1) * amount_in as f64
                / (u64::pow(10, sap_pool_mint_account.decimals as u32) as f64)
                * (performance_fee as f64 / PERCENT_MUL);
        } else {
            performance_fee_value = 0.0;
        }
        // msg!(
        //     "burn fee:{}, perforamce fee:{}",
        //     burn_fee_value,
        //     performance_fee_value
        // );
        this_sap_pool.fee += burn_fee_value;
        this_sap_pool.performance_fee += performance_fee_value;
        AmmPool::pack(this_sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        // calculate transfer amount
        let this_token_mint_account = Self::unpack_mint(&this_token_mint_acc, &token_program_id)?;
        let this_token_vault_account =
            Self::unpack_token_account(&this_token_vault_acc, &token_program_id)?;
        let amount_out =
            (burn_value - burn_fee_value - performance_fee_value) / selected_token_price;
        let transfer_amount =
            amount_out * (u64::pow(10, this_token_mint_account.decimals as u32) as f64);
        // msg!("transfer amount:{}", transfer_amount);
        if this_token_vault_account.amount < transfer_amount as u64 {
            return Err(AmmError::InsufficientFunds.into());
        }
        // transfer token to user account
        Self::token_transfer(
            sap_pool_acc.key,
            token_program_acc.clone(),
            this_token_vault_acc.clone(),
            user_token_acc.clone(),
            manager_acc.clone(),
            this_sap_pool.nonce as u8,
            transfer_amount as u64,
        )?;
        // burn sap token
        Self::token_burn(
            sap_pool_acc.key,
            token_program_acc.clone(),
            user_sap_acc.clone(),
            sap_pool_mint_acc.clone(),
            user_wallet_acc.clone(),
            this_sap_pool.nonce as u8,
            amount_in,
        )?;

        Ok(())
    }

    /// Processes `Swap` instruction.
    fn process_swap(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        if token_amount == 0 {
            return Err(AmmError::InvalidInput.into());
        }
        const FIX_NUM: usize = 8;
        if accounts.len() != (FIX_NUM + TOKEN_NUM * LIST_NUM) {
            return Err(AmmError::WrongAccountsNumber.into());
        }

        let accounts = array_ref![accounts, 0, FIX_NUM + TOKEN_NUM * LIST_NUM];
        let (fixed_accs, groups_accs) = array_refs![accounts, FIX_NUM, TOKEN_NUM * LIST_NUM];

        // let [token_program_acc, _clock_acc, etf_pool_acc, _flux_aggregator_acc, authority_acc, etf_mdi_mint_acc, user_mdi_acc, user_wallet_acc] =
        //     fixed_accs;

        let [_owner_acc, authority_acc, token_program_acc, sap_pool_acc, _sap_pool_mint_acc, manager_acc, _oracle_acc, _clock_acc] =
            fixed_accs;

        if !manager_acc.is_signer {
            return Err(AmmError::InvalidSignAccount.into());
        }

        let this_sap_pool = AmmPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;

        if *manager_acc.key != this_sap_pool.manager_pubkey {
            return Err(AmmError::InvalidManager.into());
        }

        if *authority_acc.key
            != Self::authority_id(program_id, sap_pool_acc.key, this_sap_pool.nonce as u8)?
        {
            return Err(AmmError::InvalidProgramAddress.into());
        }
        if *token_program_acc.key != spl_token::ID {
            return Err(AmmError::InvalidTokenProgramId.into());
        }

        let token_program_id = *token_program_acc.key;

        // let sap_pool_mint_account = Self::unpack_mint(sap_pool_mint_acc, &token_program_id)?;

        // check user token acc mint belongs to one of the token list
        if !(&this_sap_pool
            .token_list
            .iter()
            .any(|v| v == &token_to_buy_pubkey))
        {
            return Err(AmmError::InvalidTokenMint.into());
        }

        if !(&this_sap_pool
            .token_list
            .iter()
            .any(|v| v == &token_to_trade_pubkey))
        {
            return Err(AmmError::InvalidTokenMint.into());
        }

        // step1: check which token vault to use
        // let mut to_trade_token_vault_acc = owner_acc.clone();
        // let mut to_buy_token_vault_acc = owner_acc.clone();
        for i in 0..TOKEN_NUM {
            let group_accs = array_ref![groups_accs, i * LIST_NUM, LIST_NUM];
            let [sap_token_mint_acc, sap_token_vault_acc, _oracle_price_acc] = group_accs;
            let sap_token_vault_account =
                Self::unpack_token_account(sap_token_vault_acc, &token_program_id)?;
            if sap_token_vault_account.mint != this_sap_pool.token_assets_vault_pubkeys[i] {
                return Err(AmmError::InvalidVault.into());
            }
            // let oracle_account = Self::unpack
            // oracle get price feed for each token
            if *sap_token_mint_acc.key == token_to_trade_pubkey {
                // to_trade_token_vault_acc = sap_token_vault_acc.clone();
            }
            if *sap_token_mint_acc.key == token_to_buy_pubkey {
                // to_buy_token_vault_acc = sap_token_vault_acc.clone();
            }
        }

        // invoke dex trade buy pair and trade amount transaction

        Ok(())
    }

    /// Check account owner is the given program
    fn check_account_owner(
        account_info: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<(), ProgramError> {
        if *program_id != *account_info.owner {
            msg!(
                "Expected account to be owned by program {}, received {}",
                program_id,
                account_info.owner
            );
            Err(ProgramError::IncorrectProgramId)
        } else {
            Ok(())
        }
    }
}

impl PrintProgramError for AmmError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            AmmError::AlreadyInUse => msg!("Error: AlreadyInUse"),
            AmmError::InvalidProgramAddress => msg!("Error: InvalidProgramAddress"),
            AmmError::InvalidOwner => msg!("Error: InvalidOwner"),
            AmmError::InvalidSupply => msg!("Error: InvalidSupply"),
            AmmError::InvalidBalance => msg!("Error: InvalidBalance"),
            AmmError::InvalidDelegate => msg!("Error: InvalidDelegate"),
            AmmError::InvalidCloseAuthority => msg!("Error: Token account has a close authority"),
            AmmError::ExpectedMint => msg!("Error: ExpectedMint"),
            AmmError::ExpectedAccount => msg!("Error: ExpectedAccount"),
            AmmError::InvalidTokenProgramId => msg!("Error: InvalidTokenProgramId"),
            AmmError::InvalidInstruction => msg!("Error: InvalidInstruction"),
            AmmError::WrongAccountsNumber => msg!("Error: WrongAccountsNumber"),
            AmmError::InvalidSignAccount => msg!("Error: InvalidSignAccount"),
            AmmError::InvalidFreezeAuthority => {
                msg!("Error: Pool token mint has a freeze authority")
            }
            AmmError::InvalidUserOwner => msg!("Error: InvalidUserOwner"),
            AmmError::InvalidVault => msg!("Error: InvalidVault"),
            AmmError::InvalidMint => msg!("Error: InvalidMint"),

            AmmError::InvalidStatus => msg!("Error: InvalidStatus"),
            AmmError::InsufficientFunds => msg!("Error: InsufficientFunds"),
            AmmError::InvalidInput => msg!("Error: InvalidInput"),
            AmmError::OutOfSlippage => msg!("Error: OutOfSlippage"),
            AmmError::BaseMintNoMatch => msg!("Error: Base Mint not match"),
            AmmError::InvalidTokenMint => msg!("Error: Trade Token Mint not match"),
            AmmError::InvalidManager => msg!("Error: Invalid manager"),
            AmmError::CannotWrite => msg!("Error: SAP account is not writable"),
            AmmError::OraclePriceMissMatch => msg!("Error: OraclePriceMissMatch"),
            AmmError::OracleProductMissMatch => msg!("Error: OracleProductMissMatch"),
            AmmError::InvalidLType => msg!("Error: InvalidLType"),
            AmmError::ManagerSypMissMatch => msg!("Error: ManagerSypMissMatch"),
            AmmError::SypVaultMissMatch => msg!("Error: SypVaultMissMatch"),
            AmmError::AmmPoolExist => msg!("Error: AmmPoolExist"),
            AmmError::AmmPoolLock => msg!("Error: AmmPoolLock"),
            AmmError::InvalidAmount => msg!("Error: Amount must be greater than zero."),
            AmmError::AmmPoolNotPreMint => msg!("Error: AmmPoolNotPreMint"),
            AmmError::PreMintLimit => msg!("Error: PreMintLimit"),
            AmmError::NoFee => msg!("Error: NoFee"),
        }
    }
}

fn get_token_price_only(pyth_price_info: &AccountInfo<'_>) -> Result<f64, ProgramError> {
    let pyth_price_data = &pyth_price_info.try_borrow_data()?;
    let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

    let org_price: f64 =
        pyth_price.agg.price as f64 / (u64::pow(10, pyth_price.expo.abs() as u32) as f64);
    Ok(org_price)
}

// public key of 11111111111111111111111111111111
const NULL_PUBKEY: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);
// public key of usdc
const USDC_PUBKEY: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array([
        198, 250, 122, 243, 190, 219, 173, 58, 61, 101, 243, 106, 171, 201, 116, 49, 177, 187, 228,
        194, 210, 246, 224, 228, 124, 166, 2, 3, 69, 47, 93, 97,
    ]);

const PERCENT_MUL: f64 = 100000.0;
