//! Program state processor

use solana_program::sysvar::Sysvar;
use {
    crate::{
        error::SapError,
        instruction::SapInstruction,
        state::{SapPool, UserSapMember, LIST_NUM, TOKEN_NUM},
    },
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
                sap_init_price,
                time_lock,
                sap_pre_mint_l1_amount,
                sap_pre_mint_l2_amount,
            } => {
                msg!("Instruction: Init");
                Self::process_initialize(
                    program_id,
                    accounts,
                    nonce,
                    sap_init_price,
                    time_lock,
                    sap_pre_mint_l1_amount,
                    sap_pre_mint_l2_amount,
                )
            }
            SapInstruction::Mint {
                token_pubkey,
                amount_in,
                ltype,
                fee,
            } => {
                msg!("Instruction: Mint");
                Self::process_mint(program_id, accounts, &token_pubkey, amount_in, ltype, fee)
            }
            SapInstruction::Redeem {
                amount_in,
                ltype,
                burn_fee,
                sap_cost,
                performance_fee,
            } => {
                msg!("Instruction: Redeem");
                Self::process_redeem(
                    program_id,
                    accounts,
                    amount_in,
                    ltype,
                    burn_fee,
                    sap_cost,
                    performance_fee,
                )
            }
            SapInstruction::Trade {
                token_to_trade_pubkey,
                token_amount,
                ask_price,
                bid_price,
                trade_type,
                token_to_buy_pubkey,
            } => {
                msg!("Instruction: Trade");
                Self::process_trade(
                    program_id,
                    accounts,
                    token_to_trade_pubkey,
                    token_amount,
                    ask_price,
                    bid_price,
                    trade_type,
                    token_to_buy_pubkey,
                )
            }
            SapInstruction::UpdateTokenList {} => {
                msg!("Instruction: Update_Token_List");
                Self::process_update_token_list(program_id, accounts)
            }
            SapInstruction::UpdateManager { manager_pubkey } => {
                msg!("Instruction: Update_manager");
                Self::process_update_manager(program_id, accounts, &manager_pubkey)
            }
            SapInstruction::CreateSapMember { nonce, ltype } => {
                msg!("Instruction: Create Sap member");
                Self::process_create_user_sap(program_id, accounts, nonce, ltype)
            }
            SapInstruction::UpdateSap {
                status,
                sap_pre_mint_l1_amount,
                sap_pre_mint_l2_amount,
            } => {
                msg!("Instruction: Update Sap");
                Self::process_update_sap(
                    program_id,
                    accounts,
                    status,
                    sap_pre_mint_l1_amount,
                    sap_pre_mint_l2_amount,
                )
            }
            SapInstruction::ClaimSap {} => {
                msg!("Instruction: Claim Sap");
                Self::process_claim_sap(program_id, accounts)
            }
            SapInstruction::StartSap {} => {
                msg!("Instruction: Start Sap");
                Self::process_start_sap(program_id, accounts)
            }
            SapInstruction::PreMintSap { amount, fee } => {
                msg!("Instruction: Pre Mint Sap");
                Self::process_pre_mint(program_id, accounts, amount, fee)
            }
            SapInstruction::PreBurnSap { amount, fee } => {
                msg!("Instruction: Pre Burn Sap");
                Self::process_pre_burn(program_id, accounts, amount, fee)
            }
            SapInstruction::UpdateStatus { status } => {
                msg!("Instruction: Update Status");
                Self::process_update_status(program_id, accounts, status)
            }
            SapInstruction::WithdrawalFee {} => {
                msg!("Instruction: Withdrawal Fee");
                Self::process_withdrawal_fee(program_id, accounts)
            }
        }
    }

    /// Unpacks a spl_token `Account`.
    pub fn unpack_token_account(
        account_info: &AccountInfo,
        token_program_id: &Pubkey,
    ) -> Result<spl_token::state::Account, SapError> {
        if account_info.owner != token_program_id {
            Err(SapError::InvalidTokenProgramId)
        } else {
            spl_token::state::Account::unpack(&account_info.data.borrow())
                .map_err(|_| SapError::ExpectedAccount)
        }
    }

    /// Unpacks a spl_token `Mint`.
    pub fn unpack_mint(
        account_info: &AccountInfo,
        token_program_id: &Pubkey,
    ) -> Result<spl_token::state::Mint, SapError> {
        if account_info.owner != token_program_id {
            Err(SapError::InvalidTokenProgramId)
        } else {
            spl_token::state::Mint::unpack(&account_info.data.borrow())
                .map_err(|_| SapError::ExpectedMint)
        }
    }

    /// Calculates the authority id by generating a program address.
    pub fn authority_id(
        program_id: &Pubkey,
        my_info: &Pubkey,
        nonce: u8,
    ) -> Result<Pubkey, SapError> {
        msg!("program id:{:?}", program_id);
        msg!("my info:{:?}, nonce:{:?}", my_info, nonce);
        let ak = Pubkey::create_program_address(&[&my_info.to_bytes()[..32], &[nonce]], program_id);
        msg!("pda:{:?}", ak);
        Err(SapError::InvalidProgramAddress)
        // .or(Err(SapError::InvalidProgramAddress))
    }

    /// Issue a spl_token `Burn` instruction.
    pub fn token_burn<'a>(
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
    pub fn token_mint_to<'a>(
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
    pub fn token_mint_to2<'a>(
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
    pub fn token_transfer<'a>(
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
    pub fn process_initialize(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        nonce: u64,
        sap_init_price: f64,
        time_lock: i64,
        sap_pre_mint_l1_amount: u64,
        sap_pre_mint_l2_amount: u64,
    ) -> ProgramResult {
        msg!(&format!("process_acc len:{}", accounts.len(),));

        const FIX_NUM: usize = 8;
        if accounts.len() != FIX_NUM + TOKEN_NUM * LIST_NUM {
            return Err(SapError::WrongAccountsNumber.into());
        }
        let accounts = array_ref![accounts, 0, FIX_NUM + TOKEN_NUM * LIST_NUM];
        let (fixed_accs, groups_accs) = array_refs![accounts, FIX_NUM, TOKEN_NUM * LIST_NUM];

        let [owner_acc, token_program_acc, sap_pool_acc, sap_pool_l1_mint_acc, sap_pool_l2_mint_acc, pre_mint_l1_pubkey_acc, pre_mint_l2_pubkey_acc, manager_acc] =
            fixed_accs;

        if !sap_pool_acc.is_writable {
            return Err(SapError::CannotWrite.into());
        }
        // use data
        let mut this_sap_pool = SapPool::unpack(&sap_pool_acc.data.borrow())?;
        // check
        if this_sap_pool.manager_pubkey != NULL_PUBKEY {
            return Err(SapError::SapPoolExist.into());
        }
        if sap_pre_mint_l1_amount == 0 {
            msg!("sap pre mint amount 1:{}", sap_pre_mint_l1_amount);
            return Err(SapError::InvalidAmount.into());
        }
        if sap_pre_mint_l2_amount == 0 {
            msg!("sap pre mint amount 2:{}", sap_pre_mint_l2_amount);
            return Err(SapError::InvalidAmount.into());
        }

        if *token_program_acc.key != spl_token::ID {
            return Err(SapError::InvalidTokenProgramId.into());
        }
        let token_program_id = *token_program_acc.key;
        let sap_pool_l1_mint_account = Self::unpack_mint(sap_pool_l1_mint_acc, &token_program_id)?;
        let sap_pool_l2_mint_account = Self::unpack_mint(sap_pool_l2_mint_acc, &token_program_id)?;
        if COption::Some(*owner_acc.key) != sap_pool_l1_mint_account.mint_authority {
            return Err(SapError::InvalidOwner.into());
        }
        if COption::Some(*owner_acc.key) != sap_pool_l2_mint_account.mint_authority {
            return Err(SapError::InvalidOwner.into());
        }

        if sap_pool_l1_mint_account.freeze_authority.is_some() {
            return Err(SapError::InvalidFreezeAuthority.into());
        }
        if sap_pool_l2_mint_account.freeze_authority.is_some() {
            return Err(SapError::InvalidFreezeAuthority.into());
        }

        // init sap pool
        // 1 is pre mint
        this_sap_pool.status = 1;
        this_sap_pool.nonce = nonce;
        this_sap_pool.owner_pubkey = *owner_acc.key;
        this_sap_pool.sap_mint_l1_pubkey = *sap_pool_l1_mint_acc.key;
        this_sap_pool.sap_mint_l2_pubkey = *sap_pool_l2_mint_acc.key;
        this_sap_pool.pre_mint_l1_pubkey = *pre_mint_l1_pubkey_acc.key;
        this_sap_pool.pre_mint_l2_pubkey = *pre_mint_l2_pubkey_acc.key;
        this_sap_pool.manager_pubkey = *manager_acc.key;
        this_sap_pool.sap_init_price = sap_init_price;
        // this_sap_pool.sap_init_ts = sap_init_ts;
        this_sap_pool.time_lock = time_lock;
        this_sap_pool.sap_pre_mint_l1_amount = sap_pre_mint_l1_amount;
        this_sap_pool.sap_pre_mint_l2_amount = sap_pre_mint_l2_amount;
        msg!(
            "init price:{}, time lock:{}, pre l1 amount:{}, pre l2 amount:{}, ",
            sap_init_price,
            time_lock,
            sap_pre_mint_l1_amount,
            sap_pre_mint_l2_amount
        );

        // init sap pool token list
        for i in 0..TOKEN_NUM {
            let group_accs = array_ref![groups_accs, i * LIST_NUM, LIST_NUM];
            let [sap_token_mint_acc, sap_token_vault_acc, oracle_price_acc] = group_accs;

            // let sap_token_mint_account = Self::unpack_mint(sap_token_mint_acc, &token_program_id)?;
            let sap_token_vault_account =
                Self::unpack_token_account(sap_token_vault_acc, &token_program_id)?;
            if *manager_acc.key != sap_token_vault_account.owner {
                return Err(SapError::InvalidOwner.into());
            }
            if sap_token_vault_account.delegate.is_some() {
                return Err(SapError::InvalidDelegate.into());
            }

            if sap_token_vault_account.close_authority.is_some() {
                return Err(SapError::InvalidCloseAuthority.into());
            }
            if *sap_token_mint_acc.key != sap_token_vault_account.mint {
                return Err(SapError::BaseMintNoMatch.into());
            }
            // init sap token and
            this_sap_pool.token_list[i] = sap_token_vault_account.mint;
            this_sap_pool.token_assets_vault_pubkeys[i] = *sap_token_vault_acc.key;
            this_sap_pool.oracle_price_pubkey_list[i] = *oracle_price_acc.key;
        }

        SapPool::pack(this_sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        Ok(())
    }

    /// Processes `Mint` instruction.
    pub fn process_mint(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        token_pubkey: &Pubkey,
        amount_in: u64,
        ltype: u64,
        fee: u64,
    ) -> ProgramResult {
        msg!(&format!(
            "amount_in:{:?}, token_pubkey:{}",
            amount_in, token_pubkey
        ));
        if amount_in == 0 {
            return Err(SapError::InvalidInput.into());
        }
        const FIX_NUM: usize = 10;
        if accounts.len() != FIX_NUM + TOKEN_NUM * LIST_NUM {
            return Err(SapError::WrongAccountsNumber.into());
        }
        let accounts = array_ref![accounts, 0, FIX_NUM + TOKEN_NUM * LIST_NUM];
        let (fixed_accs, groups_accs) = array_refs![accounts, FIX_NUM, TOKEN_NUM * LIST_NUM];

        // fixed_accs;
        let [owner_acc, token_program_acc, sap_pool_acc, sap_pool_l1_mint_acc, sap_pool_l2_mint_acc, user_sap_acc, user_wallet_acc, user_token_acc, clock_acc, _chainlink_program_acc] =
            fixed_accs;
        if !user_wallet_acc.is_signer {
            return Err(SapError::InvalidSignAccount.into());
        }
        let mut this_sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        let clock = Clock::from_account_info(&clock_acc)?;
        // check
        // 3 is free mint burn
        if this_sap_pool.status != 3 {
            return Err(SapError::SapPoolLock.into());
        }
        if this_sap_pool.sap_mint_l1_pubkey != *sap_pool_l1_mint_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        if this_sap_pool.sap_mint_l2_pubkey != *sap_pool_l2_mint_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        if *token_program_acc.key != spl_token::ID {
            return Err(SapError::InvalidTokenProgramId.into());
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
                return Err(SapError::InvalidMint.into());
            }
        } else if ltype == 2 {
            sap_pool_mint_account = sap_pool_l2_mint_account.clone();
            sap_pool_mint_acc = sap_pool_l2_mint_acc.clone();
            if user_sap_account.mint != this_sap_pool.sap_mint_l2_pubkey {
                return Err(SapError::InvalidMint.into());
            }
        } else {
            return Err(SapError::InvalidMint.into());
        }
        // let sap_pool_mint_account = Self::unpack_mint(&sap_pool_mint_acc, &token_program_id)?;
        // check account
        if user_sap_account.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_token_account.amount < amount_in {
            return Err(SapError::InsufficientFunds.into());
        }
        if user_token_account.mint != *token_pubkey {
            return Err(SapError::InvalidMint.into());
        }

        // check user token acc mint belongs to one of the token list
        if !(&this_sap_pool
            .token_list
            .iter()
            .any(|v| v == &user_token_account.mint))
        {
            return Err(SapError::BaseMintNoMatch.into());
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
                return Err(SapError::InvalidVault.into());
            }
            if &this_sap_pool.oracle_price_pubkey_list[i] != oracle_price_acc.key {
                return Err(SapError::OraclePriceMissMatch.into());
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
            return Err(SapError::InvalidMint.into());
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
        SapPool::pack(this_sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
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

    /// Processes `Redeem` instruction.
    pub fn process_redeem(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount_in: u64,
        ltype: u64,
        burn_fee: u64,
        sap_cost: u64,
        performance_fee: u64,
    ) -> ProgramResult {
        msg!(&format!("amount_in:{:?}", amount_in));
        // use account
        const FIX_NUM: usize = 10;
        if accounts.len() != FIX_NUM + TOKEN_NUM * LIST_NUM {
            return Err(SapError::WrongAccountsNumber.into());
        }
        let accounts = array_ref![accounts, 0, FIX_NUM + TOKEN_NUM * LIST_NUM];
        let (fixed_accs, groups_accs) = array_refs![accounts, FIX_NUM, TOKEN_NUM * LIST_NUM];
        let [token_program_acc, sap_pool_acc, sap_pool_l1_mint_acc, sap_pool_l2_mint_acc, user_sap_acc, user_wallet_acc, user_token_acc, manager_acc, clock_acc, _chainlink_program_acc] =
            fixed_accs;

        // use data
        let mut this_sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        let clock = Clock::from_account_info(&clock_acc)?;
        // check
        // 3 is free mint burn
        if this_sap_pool.status != 3 {
            return Err(SapError::SapPoolLock.into());
        }
        if this_sap_pool.sap_mint_l1_pubkey != *sap_pool_l1_mint_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        if this_sap_pool.sap_mint_l2_pubkey != *sap_pool_l2_mint_acc.key {
            return Err(SapError::InvalidMint.into());
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
                return Err(SapError::InvalidMint.into());
            }
        } else if ltype == 2 {
            sap_pool_mint_account = sap_pool_l2_mint_account.clone();
            sap_pool_mint_acc = sap_pool_l2_mint_acc.clone();
            if user_sap_account.mint != this_sap_pool.sap_mint_l2_pubkey {
                return Err(SapError::InvalidMint.into());
            }
        } else {
            return Err(SapError::InvalidMint.into());
        }

        // check signer
        if !user_wallet_acc.is_signer {
            return Err(SapError::InvalidSignAccount.into());
        }
        // check account
        if amount_in == 0 {
            return Err(SapError::InvalidInput.into());
        }
        if *token_program_acc.key != spl_token::ID {
            return Err(SapError::InvalidTokenProgramId.into());
        }
        if user_sap_account.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_token_account.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_sap_account.amount < amount_in {
            return Err(SapError::InsufficientFunds.into());
        }
        // check user token acc mint belongs to one of the token list
        if !(&this_sap_pool
            .token_list
            .iter()
            .any(|v| v == &user_token_account.mint))
        {
            return Err(SapError::BaseMintNoMatch.into());
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
                return Err(SapError::InvalidVault.into());
            }
            if &this_sap_pool.oracle_price_pubkey_list[i] != oracle_price_acc.key {
                return Err(SapError::OraclePriceMissMatch.into());
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
            return Err(SapError::InvalidMint.into());
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
        SapPool::pack(this_sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
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
            return Err(SapError::InsufficientFunds.into());
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

    /// Processes `Trade` instruction.
    pub fn process_trade(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        token_to_trade_pubkey: Pubkey,
        token_amount: u64,
        _ask_price: u64,
        _bid_price: u64,
        _trade_type: u64,
        token_to_buy_pubkey: Pubkey,
    ) -> ProgramResult {
        if token_amount == 0 {
            return Err(SapError::InvalidInput.into());
        }
        const FIX_NUM: usize = 8;
        if accounts.len() != (FIX_NUM + TOKEN_NUM * LIST_NUM) {
            return Err(SapError::WrongAccountsNumber.into());
        }

        let accounts = array_ref![accounts, 0, FIX_NUM + TOKEN_NUM * LIST_NUM];
        let (fixed_accs, groups_accs) = array_refs![accounts, FIX_NUM, TOKEN_NUM * LIST_NUM];

        // let [token_program_acc, _clock_acc, etf_pool_acc, _flux_aggregator_acc, authority_acc, etf_mdi_mint_acc, user_mdi_acc, user_wallet_acc] =
        //     fixed_accs;

        let [_owner_acc, authority_acc, token_program_acc, sap_pool_acc, _sap_pool_mint_acc, manager_acc, _oracle_acc, _clock_acc] =
            fixed_accs;

        if !manager_acc.is_signer {
            return Err(SapError::InvalidSignAccount.into());
        }

        let this_sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;

        if *manager_acc.key != this_sap_pool.manager_pubkey {
            return Err(SapError::InvalidManager.into());
        }

        if *authority_acc.key
            != Self::authority_id(program_id, sap_pool_acc.key, this_sap_pool.nonce as u8)?
        {
            return Err(SapError::InvalidProgramAddress.into());
        }
        if *token_program_acc.key != spl_token::ID {
            return Err(SapError::InvalidTokenProgramId.into());
        }

        let token_program_id = *token_program_acc.key;

        // let sap_pool_mint_account = Self::unpack_mint(sap_pool_mint_acc, &token_program_id)?;

        // check user token acc mint belongs to one of the token list
        if !(&this_sap_pool
            .token_list
            .iter()
            .any(|v| v == &token_to_buy_pubkey))
        {
            return Err(SapError::InvalidTokenMint.into());
        }

        if !(&this_sap_pool
            .token_list
            .iter()
            .any(|v| v == &token_to_trade_pubkey))
        {
            return Err(SapError::InvalidTokenMint.into());
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
                return Err(SapError::InvalidVault.into());
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

    /// Processes update_token_list
    fn process_update_token_list(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        // token_list: [Pubkey; TOKEN_NUM],
        // token_assets_vault_pubkeys: [Pubkey; TOKEN_NUM],
    ) -> ProgramResult {
        const FIX_NUM: usize = 7;
        if accounts.len() != TOKEN_NUM * LIST_NUM + FIX_NUM {
            return Err(SapError::WrongAccountsNumber.into());
        }
        let accounts = array_ref![accounts, 0, TOKEN_NUM * LIST_NUM + FIX_NUM];
        let (fixed_accs, groups_accs) = array_refs![accounts, FIX_NUM, TOKEN_NUM * LIST_NUM];

        let [_owner_acc, _authority_acc, token_program_acc, sap_pool_acc, _sap_pool_mint_acc, manager_acc, _clock_acc] =
            fixed_accs;

        if !manager_acc.is_signer {
            return Err(SapError::InvalidSignAccount.into());
        }

        let mut this_sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        let token_program_id = *token_program_acc.key;
        if this_sap_pool.manager_pubkey != *manager_acc.key {
            return Err(SapError::InvalidSignAccount.into());
        }

        for i in 0..TOKEN_NUM {
            let group_accs = array_ref![groups_accs, i * LIST_NUM, LIST_NUM];

            // let [sap_token_mint_acc, sap_token_vault_acc,oracle_product_acc,oracle_price_acc] = group_accs;
            let [sap_token_mint_acc, sap_token_vault_acc, _oracle_price_acc] = group_accs;
            let vault_account = Self::unpack_token_account(sap_token_vault_acc, &token_program_id)?;
            msg!(
                "{:?},{:?} {:?}",
                vault_account.mint,
                sap_token_mint_acc.key,
                i
            );
            if vault_account.mint != *sap_token_mint_acc.key {
                return Err(SapError::InvalidTokenMint.into());
            }

            // if vaultAccount.owner != *sap_pool_acc.key{
            //     return Err(SapError::InvalidOwner.into());
            // }
        }
        // also check if old accounts balance equals to zero
        let mut oracle_list: [Pubkey; TOKEN_NUM] = Default::default();
        let mut token_list: [Pubkey; TOKEN_NUM] = Default::default();
        let mut vault_list: [Pubkey; TOKEN_NUM] = Default::default();
        for i in 0..TOKEN_NUM {
            let group_accs = array_ref![groups_accs, i * LIST_NUM, LIST_NUM];
            // let [sap_token_mint_acc, sap_token_vault_acc,oracle_product_acc,oracle_price_acc] = group_accs;
            let [sap_token_mint_acc, sap_token_vault_acc, oracle_price_acc] = group_accs;

            // let OldvaultAccount = Self::unpack_token_account(sap_token_vault_acc,&token_program_id)?;
            // if OldvaultAccount.amount != 0 {
            //     return Err(SapError::InvalidBalance.into());
            // }
            oracle_list[i] = *oracle_price_acc.key;
            token_list[i] = *sap_token_mint_acc.key;
            vault_list[i] = *sap_token_vault_acc.key;
        }

        this_sap_pool.token_list = token_list;
        this_sap_pool.token_assets_vault_pubkeys = vault_list;
        this_sap_pool.oracle_price_pubkey_list = oracle_list;
        SapPool::pack(this_sap_pool, &mut sap_pool_acc.data.borrow_mut())?;

        Ok(())
    }

    /// Processes update manager
    fn process_update_manager(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        manager: &Pubkey,
    ) -> ProgramResult {
        const FIX_NUM: usize = 5;
        if accounts.len() != FIX_NUM {
            return Err(SapError::WrongAccountsNumber.into());
        }
        let fixed_accs = array_ref![accounts, 0, FIX_NUM];

        let [owner_acc, sap_pool_acc, authority_acc, _manager_acc, _new_manager_acc] = fixed_accs;

        if !owner_acc.is_signer {
            return Err(SapError::InvalidSignAccount.into());
        }

        let mut this_sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;

        if *authority_acc.key
            != Self::authority_id(program_id, sap_pool_acc.key, this_sap_pool.nonce as u8)?
        {
            return Err(SapError::InvalidProgramAddress.into());
        }

        this_sap_pool.manager_pubkey = *manager;
        SapPool::pack(this_sap_pool, &mut sap_pool_acc.data.borrow_mut())?;

        Ok(())
    }

    /// Processes update manager
    pub fn process_create_user_sap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        nonce: u64,
        ltype: u64,
    ) -> ProgramResult {
        // use account
        let accounts = array_ref![accounts, 0, 6];
        let [sap_pool_acc, user_sap_member_acc, user_wallet_acc, user_sap_acc, user_reward_spt_acc, token_program_acc] =
            accounts;
        let token_program_id = *token_program_acc.key;
        // let sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        // msg!("sap pool unpack");
        msg!("user sap member {:?}", *user_sap_member_acc.key);
        let mut user_sap_member = UserSapMember::unpack(&user_sap_member_acc.data.borrow_mut())?;
        msg!("user sap member unpack");
        let user_sap = Self::unpack_token_account(user_sap_acc, &token_program_id)?;
        msg!("user sap account unpack");
        let user_reward_spt = Self::unpack_token_account(user_reward_spt_acc, &token_program_id)?;
        msg!("user reward spt account unpack");
        // check signer
        if !user_wallet_acc.is_signer {
            return Err(SapError::InvalidSignAccount.into());
        }
        // check accounts
        if user_sap_member.user_wallet != NULL_PUBKEY {
            return Err(SapError::AlreadyInUse.into());
        }
        if user_sap.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_reward_spt.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        Self::check_account_owner(sap_pool_acc, program_id)?;
        if ltype != 1 && ltype != 2 {
            return Err(SapError::InvalidLType.into());
        }
        // pack user sap model
        user_sap_member.sap_pool = *sap_pool_acc.key;
        user_sap_member.user_wallet = *user_wallet_acc.key;
        user_sap_member.user_sap_account = *user_sap_acc.key;
        user_sap_member.user_reward_spt = *user_reward_spt_acc.key;
        user_sap_member.nonce = nonce;
        user_sap_member.ltype = ltype;
        UserSapMember::pack(user_sap_member, &mut user_sap_member_acc.data.borrow_mut())?;
        msg!("user sap member pack");
        Ok(())
    }

    pub fn process_update_sap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        status: u64,
        sap_pre_mint_l1_amount: u64,
        sap_pre_mint_l2_amount: u64,
    ) -> ProgramResult {
        // use account
        let accounts = array_ref![accounts, 0, 2];
        let [sap_pool_acc, manager_acc] = accounts;
        let mut sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow_mut())?;
        // check signer
        if !manager_acc.is_signer {
            return Err(SapError::InvalidSignAccount.into());
        }
        Self::check_account_owner(sap_pool_acc, program_id)?;
        if sap_pool.manager_pubkey != *manager_acc.key {
            return Err(SapError::InvalidOwner.into());
        }
        if sap_pool.status != 2 && sap_pool.status != 3 {
            return Err(SapError::InvalidStatus.into());
        }
        if status != 2 && status != 3 {
            return Err(SapError::InvalidStatus.into());
        }
        if sap_pool.status == status {
            return Err(SapError::InvalidStatus.into());
        }
        sap_pool.status = status;
        sap_pool.sap_pre_mint_l1_amount = sap_pre_mint_l1_amount;
        sap_pool.sap_pre_mint_l2_amount = sap_pre_mint_l2_amount;
        msg!(
            "status:{}, l1 amount:{}, l2 amount:{}",
            status,
            sap_pre_mint_l1_amount,
            sap_pre_mint_l2_amount
        );
        SapPool::pack(sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        Ok(())
    }

    pub fn process_update_status(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        status: u64,
    ) -> ProgramResult {
        // use account
        let accounts = array_ref![accounts, 0, 2];
        let [sap_pool_acc, manager_acc] = accounts;
        let mut sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow_mut())?;
        // check signer
        if !manager_acc.is_signer {
            msg!("manager must sign");
            return Err(SapError::InvalidSignAccount.into());
        }
        Self::check_account_owner(sap_pool_acc, program_id)?;
        if sap_pool.manager_pubkey != *manager_acc.key {
            msg!("manager not match");
            return Err(SapError::InvalidOwner.into());
        }
        if sap_pool.status != 2 && sap_pool.status != 3 {
            msg!("status:{}", sap_pool.status);
            return Err(SapError::InvalidStatus.into());
        }
        if status != 2 && status != 3 {
            msg!("status input:{}", status);
            return Err(SapError::InvalidStatus.into());
        }
        if sap_pool.status == status {
            msg!("status is same");
            return Err(SapError::InvalidStatus.into());
        }
        sap_pool.status = status;
        msg!("status:{}", status);
        SapPool::pack(sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        Ok(())
    }

    /// Processes withdrawal fee
    fn process_withdrawal_fee(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 7];
        let [sap_pool_acc, manager_acc, vault_acc, mint_acc, owner_token_acc, user_token_acc, token_program_acc] =
            accounts;
        let mut this_sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        let token_program_id = *token_program_acc.key;
        let mint_account = Self::unpack_mint(&mint_acc, &token_program_id)?;

        // chech signer
        if !manager_acc.is_signer {
            msg!("manager must sign");
            return Err(SapError::InvalidSignAccount.into());
        }
        // calculate amount
        let fee = this_sap_pool.fee + this_sap_pool.performance_fee;
        msg!(
            "fee:{}, performance fee:{}",
            this_sap_pool.fee,
            this_sap_pool.performance_fee
        );
        if fee == 0.0 {
            return Err(SapError::NoFee.into());
        }
        // check vault
        // could only use usdc vault
        if this_sap_pool.token_assets_vault_pubkeys[0] != *vault_acc.key {
            return Err(SapError::InvalidVault.into());
        }
        if this_sap_pool.token_list[0] != *mint_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        // transfer
        let transfer_amount1 =
            this_sap_pool.fee * (u64::pow(10, mint_account.decimals as u32) as f64);
        Self::token_transfer(
            sap_pool_acc.key,
            token_program_acc.clone(),
            vault_acc.clone(),
            owner_token_acc.clone(),
            manager_acc.clone(),
            this_sap_pool.nonce as u8,
            transfer_amount1 as u64,
        )?;
        let transfer_amount2 =
            this_sap_pool.performance_fee * (u64::pow(10, mint_account.decimals as u32) as f64);
        Self::token_transfer(
            sap_pool_acc.key,
            token_program_acc.clone(),
            vault_acc.clone(),
            user_token_acc.clone(),
            manager_acc.clone(),
            this_sap_pool.nonce as u8,
            transfer_amount2 as u64,
        )?;
        // clear fee
        this_sap_pool.fee = 0.0;
        this_sap_pool.performance_fee = 0.0;
        SapPool::pack(this_sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        Ok(())
    }

    pub fn process_claim_sap(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        // use account
        let accounts = array_ref![accounts, 0, 12];
        let [sap_pool_acc, sap_pool_mint_acc, sap_pool_vault_acc, sap_pre_mint_acc, user_wallet_acc, user_sap_acc, user_pre_sap_acc, owner_acc, manager_acc, user_token_acc, user_token_mint_acc, token_program_acc] =
            accounts;
        let mut sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        let token_program_id = *token_program_acc.key;
        // check signer
        if !user_wallet_acc.is_signer {
            msg!("user wallet must sign");
            return Err(SapError::InvalidSignAccount.into());
        }
        if !owner_acc.is_signer {
            msg!("owner must sign");
            return Err(SapError::InvalidSignAccount.into());
        }
        if !manager_acc.is_signer {
            msg!("manager must sign");
            return Err(SapError::InvalidSignAccount.into());
        }
        // check
        if sap_pool.status != 2 && sap_pool.status != 3 {
            msg!("sap pool status:{}", sap_pool.status);
            return Err(SapError::InvalidStatus.into());
        }
        let user_sap = Self::unpack_token_account(user_sap_acc, &token_program_id)?;
        let user_pre_sap = Self::unpack_token_account(user_pre_sap_acc, &token_program_id)?;
        let user_token = Self::unpack_token_account(user_token_acc, &token_program_id)?;
        // let sap_pool_vault = Self::unpack_token_account(sap_pool_vault_acc, &token_program_id)?;
        let sap_pool_mint = Self::unpack_mint(sap_pool_mint_acc, &token_program_id)?;
        let sap_pre_mint = Self::unpack_mint(sap_pre_mint_acc, &token_program_id)?;
        let user_token_mint = Self::unpack_mint(user_token_mint_acc, &token_program_id)?;
        // check accounts
        let ltype: u8;
        if sap_pool.pre_mint_l1_pubkey == *sap_pre_mint_acc.key {
            ltype = 1;
            if sap_pool.sap_mint_l1_pubkey != *sap_pool_mint_acc.key {
                msg!("sap l1 mint not match");
                return Err(SapError::InvalidMint.into());
            }
        } else if sap_pool.pre_mint_l2_pubkey == *sap_pre_mint_acc.key {
            ltype = 2;
            if sap_pool.sap_mint_l2_pubkey != *sap_pool_mint_acc.key {
                msg!("sap l2 mint not match");
                return Err(SapError::InvalidMint.into());
            }
        } else {
            msg!("pre mint not match");
            return Err(SapError::InvalidMint.into());
        }
        if user_sap.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_pre_sap.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_token.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if sap_pool.token_list[0] != *user_token_mint_acc.key {
            msg!(
                "must use usdc {} {}",
                sap_pool.token_list[0],
                *user_token_mint_acc.key
            );
            return Err(SapError::InvalidMint.into());
        }
        if sap_pool.token_assets_vault_pubkeys[0] != *sap_pool_vault_acc.key {
            msg!(
                "must use usdc {} {}",
                sap_pool.token_assets_vault_pubkeys[0],
                *sap_pool_vault_acc.key
            );
            return Err(SapError::InvalidVault.into());
        }
        Self::check_account_owner(sap_pool_acc, program_id)?;
        // use data
        let this_claim_rate: f64;
        if ltype == 1 {
            this_claim_rate = sap_pool.sap_pre_mint_l1_claim_rate;
        } else if ltype == 2 {
            this_claim_rate = sap_pool.sap_pre_mint_l2_claim_rate;
        } else {
            msg!("ltype:{}", ltype);
            return Err(SapError::InvalidMint.into());
        }
        // update sap pool
        let update_amount: u64 = ((user_pre_sap.amount as f64) * this_claim_rate).round() as u64;
        msg!(
            "amount:{}, rate:{}, update amount:{}",
            user_pre_sap.amount,
            this_claim_rate,
            update_amount
        );
        if ltype == 1 {
            sap_pool.sap_pre_mint_l1_amount = sap_pool
                .sap_pre_mint_l1_amount
                .checked_sub(update_amount)
                .unwrap();
            msg!(
                "data:{}, user pre sap amount:{}",
                sap_pool.sap_pre_mint_l1_amount,
                user_pre_sap.amount
            );
        } else if ltype == 2 {
            sap_pool.sap_pre_mint_l2_amount = sap_pool
                .sap_pre_mint_l2_amount
                .checked_sub(update_amount)
                .unwrap();
            msg!(
                "data:{}, user pre sap amount:{}",
                sap_pool.sap_pre_mint_l2_amount,
                user_pre_sap.amount
            );
        } else {
            msg!("ltype:{}", ltype);
            return Err(SapError::InvalidMint.into());
        }
        SapPool::pack(sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        // calculate amount
        let pre_value =
            (user_pre_sap.amount as f64) / (u64::pow(10, sap_pre_mint.decimals as u32) as f64);
        let claim_amount = pre_value * this_claim_rate;
        let back_amount = pre_value * (1.0 - this_claim_rate) * sap_pool.sap_init_price;
        msg!("pre value:{}, rate:{}", pre_value, this_claim_rate);
        msg!("claim amount:{}, back_amount:{}", claim_amount, back_amount);
        if claim_amount == 0.0 {
            return Err(SapError::InvalidMint.into());
        }
        // mint sap token
        {
            let amount_mint =
                (claim_amount * u64::pow(10, sap_pool_mint.decimals as u32) as f64) as u64;
            Self::token_mint_to(
                sap_pool_acc.key,
                token_program_acc.clone(),
                sap_pool_mint_acc.clone(),
                user_sap_acc.clone(),
                owner_acc.clone(),
                sap_pool.nonce as u8,
                amount_mint,
            )?;
        }
        if back_amount > 0.0 {
            // transfer back token
            let amount_back =
                (back_amount * u64::pow(10, user_token_mint.decimals as u32) as f64) as u64;
            Self::token_transfer(
                sap_pool_acc.key,
                token_program_acc.clone(),
                sap_pool_vault_acc.clone(),
                user_token_acc.clone(),
                manager_acc.clone(),
                sap_pool.nonce as u8,
                amount_back as u64,
            )?;
        }
        // burn pre token
        Self::token_burn(
            sap_pool_acc.key,
            token_program_acc.clone(),
            user_pre_sap_acc.clone(),
            sap_pre_mint_acc.clone(),
            user_wallet_acc.clone(),
            sap_pool.nonce as u8,
            user_pre_sap.amount,
        )?;

        Ok(())
    }

    pub fn process_start_sap(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        // use account
        let accounts = array_ref![accounts, 0, 6];
        let [sap_pool_acc, manager_acc, pre_mint_l1_acc, pre_mint_l2_acc, clock_acc, token_program_acc] =
            accounts;
        let mut sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        let token_program_id = *token_program_acc.key;
        // check signer
        if !manager_acc.is_signer {
            msg!("manager must sign");
            return Err(SapError::InvalidSignAccount.into());
        }
        if sap_pool.pre_mint_l1_pubkey != *pre_mint_l1_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        if sap_pool.pre_mint_l2_pubkey != *pre_mint_l2_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        Self::check_account_owner(sap_pool_acc, program_id)?;
        // update status
        if sap_pool.manager_pubkey != *manager_acc.key {
            return Err(SapError::InvalidOwner.into());
        }
        if sap_pool.status != 1 {
            msg!("sap pool status:{}", sap_pool.status);
            return Err(SapError::InvalidStatus.into());
        }
        sap_pool.status = 3;
        // update start time
        let clock = Clock::from_account_info(&clock_acc)?;
        sap_pool.sap_init_ts = clock.unix_timestamp;
        // update l1 l2 amount
        let target_rate: f64 =
            sap_pool.sap_pre_mint_l1_amount as f64 / sap_pool.sap_pre_mint_l2_amount as f64;
        msg!(
            "amount1:{}, amount2:{}, target rate:{}",
            sap_pool.sap_pre_mint_l1_amount,
            sap_pool.sap_pre_mint_l2_amount,
            target_rate
        );
        let pre_mint_l1_account = Self::unpack_mint(pre_mint_l1_acc, &token_program_id)?;
        let pre_mint_l2_account = Self::unpack_mint(pre_mint_l2_acc, &token_program_id)?;
        let current_rate: f64 =
            pre_mint_l1_account.supply as f64 / pre_mint_l2_account.supply as f64;
        msg!(
            "amount1:{}, amount2:{}, current rate:{}",
            pre_mint_l1_account.supply,
            pre_mint_l2_account.supply,
            current_rate
        );
        if target_rate > current_rate {
            sap_pool.sap_pre_mint_l1_amount = pre_mint_l1_account.supply;
            sap_pool.sap_pre_mint_l2_amount =
                (pre_mint_l1_account.supply as f64 / target_rate).round() as u64;
            msg!(
                "amount1:{}, amount2:{}",
                sap_pool.sap_pre_mint_l1_amount,
                sap_pool.sap_pre_mint_l2_amount
            );
            sap_pool.sap_pre_mint_l1_claim_rate = 1.0;
            sap_pool.sap_pre_mint_l2_claim_rate =
                sap_pool.sap_pre_mint_l2_amount as f64 / pre_mint_l2_account.supply as f64;
            msg!(
                "rate1:{}, rate2:{}",
                sap_pool.sap_pre_mint_l1_claim_rate,
                sap_pool.sap_pre_mint_l2_claim_rate
            );
        } else {
            sap_pool.sap_pre_mint_l2_amount = pre_mint_l2_account.supply;
            sap_pool.sap_pre_mint_l1_amount =
                (pre_mint_l2_account.supply as f64 * target_rate).round() as u64;
            msg!(
                "amount1:{}, amount2:{}",
                sap_pool.sap_pre_mint_l1_amount,
                sap_pool.sap_pre_mint_l2_amount
            );
            sap_pool.sap_pre_mint_l2_claim_rate = 1.0;
            sap_pool.sap_pre_mint_l1_claim_rate =
                sap_pool.sap_pre_mint_l1_amount as f64 / pre_mint_l1_account.supply as f64;
            msg!(
                "rate1:{}, rate2:{}",
                sap_pool.sap_pre_mint_l1_claim_rate,
                sap_pool.sap_pre_mint_l2_claim_rate
            );
        }
        SapPool::pack(sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        Ok(())
    }

    pub fn process_pre_mint(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        fee: u64,
    ) -> ProgramResult {
        // use account
        let accounts = array_ref![accounts, 0, 10];
        let [sap_pool_acc, sap_pre_mint_acc, sap_pool_vault_acc, oracle_price_acc, user_wallet_acc, user_token_acc, user_sap_acc, user_token_mint_acc, owner_acc, token_program_acc] =
            accounts;
        let token_program_id = *token_program_acc.key;
        let mut sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        // check
        if sap_pool.status != 1 {
            return Err(SapError::InvalidStatus.into());
        }
        let sap_pre_mint = Self::unpack_mint(sap_pre_mint_acc, &token_program_id)?;
        let user_token_mint = Self::unpack_mint(user_token_mint_acc, &token_program_id)?;
        let sap_pool_vault = Self::unpack_token_account(sap_pool_vault_acc, &token_program_id)?;
        let user_token = Self::unpack_token_account(user_token_acc, &token_program_id)?;
        let user_sap = Self::unpack_token_account(user_sap_acc, &token_program_id)?;
        // check signer
        if !user_wallet_acc.is_signer {
            return Err(SapError::InvalidSignAccount.into());
        }
        if !owner_acc.is_signer {
            return Err(SapError::InvalidSignAccount.into());
        }
        // check accounts
        if user_sap.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_token.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_token.mint != sap_pool_vault.mint {
            return Err(SapError::InvalidMint.into());
        }
        if user_token.mint != *user_token_mint_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        if sap_pool.pre_mint_l1_pubkey == *sap_pre_mint_acc.key {
            if sap_pre_mint.supply + amount > sap_pool.sap_pre_mint_l1_amount {
                return Err(SapError::PreMintLimit.into());
            }
        } else if sap_pool.pre_mint_l2_pubkey == *sap_pre_mint_acc.key {
            if sap_pre_mint.supply + amount > sap_pool.sap_pre_mint_l2_amount {
                return Err(SapError::PreMintLimit.into());
            }
        } else {
            return Err(SapError::InvalidMint.into());
        }
        if sap_pool.token_assets_vault_pubkeys[0] != *sap_pool_vault_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        if sap_pool.oracle_price_pubkey_list[0] != *oracle_price_acc.key {
            return Err(SapError::OraclePriceMissMatch.into());
        }
        Self::check_account_owner(sap_pool_acc, program_id)?;
        // calculate value
        let mint_value = amount as f64 * sap_pool.sap_init_price
            / (u64::pow(10, sap_pre_mint.decimals as u32) as f64);
        // calculate fee
        let mint_fee = fee as f64 / PERCENT_MUL;
        let mint_fee_value = mint_value * mint_fee;
        msg!("mint value:{}, fee:{}", mint_value, mint_fee_value);
        // save fee
        sap_pool.fee += mint_fee_value;
        SapPool::pack(sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        // calculate transfer amount
        let token_price: f64 = get_token_price_only(oracle_price_acc)?;
        let amount_out = (mint_value + mint_fee_value) / token_price;
        let transfer_amount = amount_out * (u64::pow(10, user_token_mint.decimals as u32) as f64);
        msg!("transfer amount:{}", transfer_amount);
        // tranfer to vault
        Self::token_transfer(
            sap_pool_acc.key,
            token_program_acc.clone(),
            user_token_acc.clone(),
            sap_pool_vault_acc.clone(),
            user_wallet_acc.clone(),
            sap_pool.nonce as u8,
            transfer_amount as u64,
        )?;
        // pre mint
        Self::token_mint_to(
            sap_pool_acc.key,
            token_program_acc.clone(),
            sap_pre_mint_acc.clone(),
            user_sap_acc.clone(),
            owner_acc.clone(),
            sap_pool.nonce as u8,
            amount,
        )?;
        Ok(())
    }

    pub fn process_pre_burn(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        fee: u64,
    ) -> ProgramResult {
        // use account
        let accounts = array_ref![accounts, 0, 10];
        let [sap_pool_acc, sap_pre_mint_acc, sap_pool_vault_acc, oracle_price_acc, user_wallet_acc, user_token_acc, user_sap_acc, user_token_mint_acc, manager_acc, token_program_acc] =
            accounts;
        let token_program_id = *token_program_acc.key;
        let mut sap_pool = SapPool::unpack_unchecked(&sap_pool_acc.data.borrow())?;
        // check
        if sap_pool.status != 1 {
            return Err(SapError::InvalidStatus.into());
        }
        let sap_pre_mint = Self::unpack_mint(sap_pre_mint_acc, &token_program_id)?;
        let user_token_mint = Self::unpack_mint(user_token_mint_acc, &token_program_id)?;
        let sap_pool_vault = Self::unpack_token_account(sap_pool_vault_acc, &token_program_id)?;
        let user_token = Self::unpack_token_account(user_token_acc, &token_program_id)?;
        let user_sap = Self::unpack_token_account(user_sap_acc, &token_program_id)?;
        // check signer
        if !user_wallet_acc.is_signer {
            msg!("user wallet must sign");
            return Err(SapError::InvalidSignAccount.into());
        }
        if !manager_acc.is_signer {
            msg!("manager must sign");
            return Err(SapError::InvalidSignAccount.into());
        }
        // check accounts
        if user_sap.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_token.owner != *user_wallet_acc.key {
            return Err(SapError::InvalidUserOwner.into());
        }
        if user_token.mint != sap_pool_vault.mint {
            return Err(SapError::InvalidMint.into());
        }
        if user_token.mint != *user_token_mint_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        if sap_pool.pre_mint_l1_pubkey != *sap_pre_mint_acc.key
            && sap_pool.pre_mint_l2_pubkey != *sap_pre_mint_acc.key
        {
            return Err(SapError::InvalidMint.into());
        }
        if sap_pool.token_assets_vault_pubkeys[0] != *sap_pool_vault_acc.key {
            return Err(SapError::InvalidMint.into());
        }
        if sap_pool.oracle_price_pubkey_list[0] != *oracle_price_acc.key {
            return Err(SapError::OraclePriceMissMatch.into());
        }
        Self::check_account_owner(sap_pool_acc, program_id)?;
        // calculate value
        let burn_value = amount as f64 * sap_pool.sap_init_price
            / (u64::pow(10, sap_pre_mint.decimals as u32) as f64);
        // calculate fee
        let burn_fee = fee as f64 / PERCENT_MUL;
        let burn_fee_value = burn_value * burn_fee;
        msg!("burn value:{}, fee:{}", burn_value, burn_fee_value);
        // save fee
        sap_pool.fee += burn_fee_value;
        SapPool::pack(sap_pool, &mut sap_pool_acc.data.borrow_mut())?;
        // calculate transfer amount
        let token_price: f64 = get_token_price_only(oracle_price_acc)?;
        let amount_out = (burn_value - burn_fee_value) / token_price;
        let transfer_amount = amount_out * (u64::pow(10, user_token_mint.decimals as u32) as f64);
        msg!("transfer amount:{}", transfer_amount);
        // tranfer from vault
        Self::token_transfer(
            sap_pool_acc.key,
            token_program_acc.clone(),
            sap_pool_vault_acc.clone(),
            user_token_acc.clone(),
            manager_acc.clone(),
            sap_pool.nonce as u8,
            transfer_amount as u64,
        )?;
        // pre burn
        Self::token_burn(
            sap_pool_acc.key,
            token_program_acc.clone(),
            user_sap_acc.clone(),
            sap_pre_mint_acc.clone(),
            user_wallet_acc.clone(),
            sap_pool.nonce as u8,
            amount,
        )?;
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
impl PrintProgramError for SapError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            SapError::AlreadyInUse => msg!("Error: AlreadyInUse"),
            SapError::InvalidProgramAddress => msg!("Error: InvalidProgramAddress"),
            SapError::InvalidOwner => msg!("Error: InvalidOwner"),
            SapError::InvalidSupply => msg!("Error: InvalidSupply"),
            SapError::InvalidBalance => msg!("Error: InvalidBalance"),
            SapError::InvalidDelegate => msg!("Error: InvalidDelegate"),
            SapError::InvalidCloseAuthority => msg!("Error: Token account has a close authority"),
            SapError::ExpectedMint => msg!("Error: ExpectedMint"),
            SapError::ExpectedAccount => msg!("Error: ExpectedAccount"),
            SapError::InvalidTokenProgramId => msg!("Error: InvalidTokenProgramId"),
            SapError::InvalidInstruction => msg!("Error: InvalidInstruction"),
            SapError::WrongAccountsNumber => msg!("Error: WrongAccountsNumber"),
            SapError::InvalidSignAccount => msg!("Error: InvalidSignAccount"),
            SapError::InvalidFreezeAuthority => {
                msg!("Error: Pool token mint has a freeze authority")
            }
            SapError::InvalidUserOwner => msg!("Error: InvalidUserOwner"),
            SapError::InvalidVault => msg!("Error: InvalidVault"),
            SapError::InvalidMint => msg!("Error: InvalidMint"),

            SapError::InvalidStatus => msg!("Error: InvalidStatus"),
            SapError::InsufficientFunds => msg!("Error: InsufficientFunds"),
            SapError::InvalidInput => msg!("Error: InvalidInput"),
            SapError::OutOfSlippage => msg!("Error: OutOfSlippage"),
            SapError::BaseMintNoMatch => msg!("Error: Base Mint not match"),
            SapError::InvalidTokenMint => msg!("Error: Trade Token Mint not match"),
            SapError::InvalidManager => msg!("Error: Invalid manager"),
            SapError::CannotWrite => msg!("Error: SAP account is not writable"),
            SapError::OraclePriceMissMatch => msg!("Error: OraclePriceMissMatch"),
            SapError::OracleProductMissMatch => msg!("Error: OracleProductMissMatch"),
            SapError::InvalidLType => msg!("Error: InvalidLType"),
            SapError::ManagerSypMissMatch => msg!("Error: ManagerSypMissMatch"),
            SapError::SypVaultMissMatch => msg!("Error: SypVaultMissMatch"),
            SapError::SapPoolExist => msg!("Error: SapPoolExist"),
            SapError::SapPoolLock => msg!("Error: SapPoolLock"),
            SapError::InvalidAmount => msg!("Error: Amount must be greater than zero."),
            SapError::SapPoolNotPreMint => msg!("Error: SapPoolNotPreMint"),
            SapError::PreMintLimit => msg!("Error: PreMintLimit"),
            SapError::NoFee => msg!("Error: NoFee"),
        }
    }
}

// fn get_price_type(ptype: &PriceType) -> &'static str {
//     match ptype {
//         PriceType::Unknown => "unknown",
//         PriceType::Price => "price",
//         // PriceType::TWAP => "twap",
//         // PriceType::Volatility => "volatility",
//     }
// }

// fn get_status(st: &PriceStatus) -> &'static str {
//     match st {
//         PriceStatus::Unknown => "unknown",
//         PriceStatus::Trading => "trading",
//         PriceStatus::Halted => "halted",
//         PriceStatus::Auction => "auction",
//     }
// }

// fn get_corp_act(cact: &CorpAction) -> &'static str {
//     match cact {
//         CorpAction::NoCorpAct => "nocorpact",
//     }
// }

// fn _get_token_price(
//     pyth_product_info: &AccountInfo<'_>,
//     pyth_price_info: &AccountInfo<'_>,
// ) -> Result<i64, ProgramError> {
//     let pyth_product_data = &pyth_product_info.try_borrow_data()?;
//     let pyth_product = pyth_client::cast::<pyth_client::Product>(pyth_product_data);

//     if pyth_product.magic != pyth_client::MAGIC {
//         msg!("Pyth product account provided is not a valid Pyth account");
//         return Err(ProgramError::InvalidArgument.into());
//     }
//     if pyth_product.atype != pyth_client::AccountType::Product as u32 {
//         msg!("Pyth product account provided is not a valid Pyth product account");
//         return Err(ProgramError::InvalidArgument.into());
//     }

//     if !pyth_product.px_acc.is_valid() {
//         msg!("Pyth product price account is invalid");
//         return Err(ProgramError::InvalidArgument.into());
//     }

//     let pyth_price_pubkey = Pubkey::new(&pyth_product.px_acc.val);
//     if &pyth_price_pubkey != pyth_price_info.key {
//         msg!("Pyth product price account does not match the Pyth price provided");
//         return Err(ProgramError::InvalidArgument.into());
//     }

//     let pyth_price_data = &pyth_price_info.try_borrow_data()?;
//     let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

//     msg!("  price_account .. {:?}", pyth_price_info.key);
//     msg!("    price_type ... {}", get_price_type(&pyth_price.ptype));
//     msg!("    exponent ..... {}", pyth_price.expo);
//     msg!("    status ....... {}", get_status(&pyth_price.agg.status));
//     msg!(
//         "    corp_act ..... {}",
//         get_corp_act(&pyth_price.agg.corp_act)
//     );
//     msg!("    price ........ {}", pyth_price.agg.price);
//     msg!("    conf ......... {}", pyth_price.agg.conf);
//     msg!("    valid_slot ... {}", pyth_price.valid_slot);
//     msg!("    publish_slot . {}", pyth_price.agg.pub_slot);
//     Ok(pyth_price.agg.price)
// }

fn find_rewards_price(
    unit_sap_price: f64,
    supply1: f64,
    supply2: f64,
    base_price: f64,
    d: f64,
) -> [f64; 2] {
    let apy = 0.2;
    let reward1 = apy / 365.0 * d;
    let reward2;
    if supply2 == 0.0 {
        reward2 = 0.0;
    } else {
        let profit = (unit_sap_price - base_price) / base_price;
        reward2 = ((supply1 + supply2) * profit - reward1 * supply1) / supply2;
    }
    return [reward1, reward2];
}

fn get_token_price_only(pyth_price_info: &AccountInfo<'_>) -> Result<f64, ProgramError> {
    let pyth_price_data = &pyth_price_info.try_borrow_data()?;
    let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

    // msg!("  price_account .. {:?}", pyth_price_info.key);
    // msg!("    price_type ... {}", get_price_type(&pyth_price.ptype));
    // msg!("    exponent ..... {}", pyth_price.expo);
    // msg!("    status ....... {}", get_status(&pyth_price.agg.status));
    // msg!(
    //     "    corp_act ..... {}",
    //     get_corp_act(&pyth_price.agg.corp_act)
    // );
    // // msg!("    price ........ {}", pyth_price.agg.price);
    // msg!("    conf ......... {}", pyth_price.agg.conf);
    // msg!("    valid_slot ... {}", pyth_price.valid_slot);
    // msg!("    publish_slot . {}", pyth_price.agg.pub_slot);
    let org_price: f64 =
        pyth_price.agg.price as f64 / (u64::pow(10, pyth_price.expo.abs() as u32) as f64);
    Ok(org_price)
}

fn _get_token_price_chain_link<'info>(
    program_acc: &AccountInfo<'info>,
    feed_acc: &AccountInfo<'info>,
) -> Result<f64, ProgramError> {
    let round;
    let decimals;
    {
        let program = program_acc.clone();
        let feed = feed_acc.clone();
        round = chainlink_solana::latest_round_data(program, feed)?;
    }
    {
        let program = program_acc.clone();
        let feed = feed_acc.clone();
        decimals = chainlink_solana::decimals(program, feed)?;
    }
    let value = round.answer;
    let price = value as f64 / (u64::pow(10, decimals as u32) as f64);
    Ok(price)
}

// public key of syp
pub const SYP_PUBKEY: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array([
        219, 159, 92, 199, 244, 36, 144, 16, 19, 35, 80, 231, 67, 217, 158, 42, 48, 240, 162, 100,
        184, 179, 69, 207, 35, 98, 47, 14, 78, 202, 157, 20,
    ]);
// public key of 11111111111111111111111111111111
pub const NULL_PUBKEY: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);
// public key of usdc
pub const USDC_PUBKEY: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array([
        198, 250, 122, 243, 190, 219, 173, 58, 61, 101, 243, 106, 171, 201, 116, 49, 177, 187, 228,
        194, 210, 246, 224, 228, 124, 166, 2, 3, 69, 47, 93, 97,
    ]);

pub const SYP_PRICE: f64 = 0.05;

pub const PERCENT_MUL: f64 = 100000.0;

pub const INIT_PRICE: f64 = 100.0;

const DAY_TIME: f64 = 24.0 * 3600.0;
