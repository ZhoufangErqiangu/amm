//! Program state processor
// use solana_program::sysvar::Sysvar;
use {
    crate::{
        error::AmmError,
        instruction::{AmmInstruction, Direction},
        state::{AmmPool, PoolStatus},
    },
    arrayref::array_ref,
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo,
        // clock::Clock,
        decode_error::DecodeError,
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::{PrintProgramError, ProgramError},
        program_memory::sol_memset,
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
        let instruction = AmmInstruction::unpack(input)?;
        match instruction {
            AmmInstruction::Initialize {
                nonce,
                fee,
                amount_a,
                amount_b,
                tolerance,
            } => {
                msg!("Instruction: Init");
                Self::process_initialize(
                    program_id, accounts, nonce, fee, amount_a, amount_b, tolerance,
                )
            }
            AmmInstruction::UpdateStatus { status } => {
                msg!("Instruction: Update Status");
                Self::process_update_status(program_id, accounts, status)
            }
            AmmInstruction::UpdateTolerance { tolerance } => {
                msg!("Instruction: Update Tolerance");
                Self::process_update_tolerance(program_id, accounts, tolerance)
            }
            AmmInstruction::Terminate {} => {
                msg!("Instruction: Terminate");
                Self::process_terminate(program_id, accounts)
            }
            AmmInstruction::Swap { amount, direction } => {
                msg!("Instruction: Swap");
                Self::process_swap(program_id, accounts, amount, direction)
            }
            AmmInstruction::WithdrawalFee {} => {
                msg!("Instruction: Withdrawal Fee");
                Self::process_withdrawal_fee(program_id, accounts)
            }
        }
    }

    /// Processes `Initialize` instruction.
    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        nonce: u8,
        fee: u64,
        amount_a: u64,
        amount_b: u64,
        tolerance: u64,
    ) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 11];
        let [pool_acc, owner_acc, mint_a_acc, mint_b_acc, vault_a_acc, vault_b_acc, fee_vault_acc, pool_pda, owner_token_a_acc, owner_token_b_acc, token_program_acc] =
            accounts;
        // use data
        let mut pool = AmmPool::unpack_unchecked(&pool_acc.data.borrow())?;
        let pda_seed = &[pool_acc.key.as_ref(), &[nonce]];
        let pda = solana_program::pubkey::Pubkey::create_program_address(pda_seed, program_id)?;
        // check
        if !owner_acc.is_signer {
            msg!("owner must sign");
            return Err(AmmError::InvalidSignAccount.into());
        }
        Self::check_account_owner(pool_acc, program_id)?;
        if pool.status != PoolStatus::NotInit {
            return Err(AmmError::PoolExist.into());
        }
        // check vault a
        let vault_a = Self::unpack_token_account(vault_a_acc)?;
        if vault_a.mint != *mint_a_acc.key {
            msg!(
                "vault a mint not match {} {}",
                vault_a.mint,
                *mint_a_acc.key
            );
            return Err(AmmError::InvalidMint.into());
        }
        if vault_a.owner != *pool_pda.key {
            msg!("vault a owner not match {} {}", vault_a.mint, *pool_pda.key);
            return Err(AmmError::InvalidOwner.into());
        }
        // check vault b
        let vault_b = Self::unpack_token_account(vault_b_acc)?;
        if vault_b.mint != *mint_b_acc.key {
            msg!(
                "vault b mint not match {} {}",
                vault_b.mint,
                *mint_b_acc.key
            );
            return Err(AmmError::InvalidMint.into());
        }
        if vault_b.owner != *pool_pda.key {
            msg!("vault b owner not match {} {}", vault_b.mint, *pool_pda.key);
            return Err(AmmError::InvalidOwner.into());
        }
        // check fee vault
        let fee_vault = Self::unpack_token_account(fee_vault_acc)?;
        if fee_vault.mint != *mint_b_acc.key {
            msg!(
                "fee vault mint not match {} {}",
                fee_vault.mint,
                *mint_b_acc.key
            );
            return Err(AmmError::InvalidMint.into());
        }
        if fee_vault.owner != *pool_pda.key {
            msg!(
                "fee vault owner not match {} {}",
                fee_vault.owner,
                *pool_pda.key
            );
            return Err(AmmError::InvalidOwner.into());
        }
        // check pda
        if pda != *pool_pda.key {
            return Err(AmmError::InvalidPDA.into());
        }
        // transfer asset to vault
        Self::token_transfer(
            token_program_acc.clone(),
            owner_token_a_acc.clone(),
            vault_a_acc.clone(),
            owner_acc.clone(),
            amount_a,
        )?;
        Self::token_transfer(
            token_program_acc.clone(),
            owner_token_b_acc.clone(),
            vault_b_acc.clone(),
            owner_acc.clone(),
            amount_b,
        )?;
        // init pool
        pool.status = PoolStatus::Nomal;
        pool.nonce = nonce;
        pool.ka = amount_a;
        pool.kb = amount_b;
        pool.tolerance = tolerance;
        pool.fee = fee;
        pool.owner = *owner_acc.key;
        pool.mint_a = *mint_a_acc.key;
        pool.mint_b = *mint_b_acc.key;
        pool.vault_a = *vault_a_acc.key;
        pool.vault_b = *vault_b_acc.key;
        pool.fee_vault = *fee_vault_acc.key;
        // pack pool
        AmmPool::pack(pool, &mut pool_acc.data.borrow_mut())?;
        Ok(())
    }

    /// Processes `Update Status` instruction.
    fn process_update_status(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        status: u8,
    ) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 2];
        let [pool_acc, owner_acc] = accounts;
        // use data
        let mut pool = AmmPool::unpack_unchecked(&pool_acc.data.borrow())?;
        // check
        if !owner_acc.is_signer {
            msg!("owner must sign");
            return Err(AmmError::InvalidSignAccount.into());
        }
        Self::check_account_owner(pool_acc, program_id)?;
        if pool.owner != *owner_acc.key {
            msg!("owner not match {} {}", pool.owner, *owner_acc.key);
            return Err(AmmError::InvalidOwner.into());
        }
        if pool.status == PoolStatus::NotInit {
            msg!("pool status:{}", pool.status);
            return Err(AmmError::InvalidStatus.into());
        }
        // update pool
        pool.status = match status {
            1 => PoolStatus::Nomal,
            2 => PoolStatus::Lock,
            _ => {
                return Err(AmmError::InvalidStatus.into());
            }
        };
        // pack pool
        AmmPool::pack(pool, &mut pool_acc.data.borrow_mut())?;
        Ok(())
    }

    /// Processes `Update Tolerance` instruction.
    fn process_update_tolerance(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        tolerance: u64,
    ) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 2];
        let [pool_acc, owner_acc] = accounts;
        // use data
        let mut pool = AmmPool::unpack_unchecked(&pool_acc.data.borrow())?;
        // check
        if !owner_acc.is_signer {
            msg!("owner must sign");
            return Err(AmmError::InvalidSignAccount.into());
        }
        Self::check_account_owner(pool_acc, program_id)?;
        if pool.owner != *owner_acc.key {
            msg!("owner not match {} {}", pool.owner, *owner_acc.key);
            return Err(AmmError::InvalidOwner.into());
        }
        if pool.status == PoolStatus::NotInit {
            msg!("pool status:{}", pool.status);
            return Err(AmmError::InvalidStatus.into());
        }
        // update pool
        pool.tolerance = tolerance;
        // pack pool
        AmmPool::pack(pool, &mut pool_acc.data.borrow_mut())?;
        Ok(())
    }

    /// Processes `Terminate` instruction.
    fn process_terminate(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 9];
        let [pool_acc, owner_acc, vault_a_acc, vault_b_acc, fee_vault_acc, pool_pda, owner_token_a_acc, owner_token_b_acc, token_program_acc] =
            accounts;
        // use data
        let pool = AmmPool::unpack_unchecked(&pool_acc.data.borrow())?;
        let vault_a = Self::unpack_token_account(vault_a_acc)?;
        let vault_b = Self::unpack_token_account(vault_b_acc)?;
        let fee_vault = Self::unpack_token_account(fee_vault_acc)?;
        // check
        if !owner_acc.is_signer {
            msg!("owner must sign");
            return Err(AmmError::InvalidSignAccount.into());
        }
        Self::check_account_owner(pool_acc, program_id)?;
        if pool.owner != *owner_acc.key {
            msg!("owner not match {} {}", pool.owner, *owner_acc.key);
            return Err(AmmError::InvalidOwner.into());
        }
        if pool.status == PoolStatus::NotInit {
            msg!("pool status:{}", pool.status);
            return Err(AmmError::InvalidStatus.into());
        }
        if pool.vault_a != *vault_a_acc.key {
            msg!("vault a not match {} {}", pool.vault_a, *vault_a_acc.key);
            return Err(AmmError::InvalidVault.into());
        }
        if pool.vault_b != *vault_b_acc.key {
            msg!("vault b not match {} {}", pool.vault_b, *vault_b_acc.key);
            return Err(AmmError::InvalidVault.into());
        }
        if pool.fee_vault != *fee_vault_acc.key {
            msg!("vault b not match {} {}", pool.vault_b, *vault_b_acc.key);
            return Err(AmmError::InvalidVault.into());
        }
        // transfer vault a
        Self::token_transfer_signed(
            pool_acc.clone(),
            pool.nonce,
            token_program_acc.clone(),
            vault_a_acc.clone(),
            owner_token_a_acc.clone(),
            pool_pda.clone(),
            vault_a.amount,
        )?;
        Self::token_close_signed(
            pool_acc.clone(),
            pool.nonce,
            token_program_acc.clone(),
            vault_a_acc.clone(),
            owner_acc.clone(),
            pool_pda.clone(),
        )?;
        // transfer vault b
        Self::token_transfer_signed(
            pool_acc.clone(),
            pool.nonce,
            token_program_acc.clone(),
            vault_b_acc.clone(),
            owner_token_b_acc.clone(),
            pool_pda.clone(),
            vault_b.amount,
        )?;
        Self::token_close_signed(
            pool_acc.clone(),
            pool.nonce,
            token_program_acc.clone(),
            vault_b_acc.clone(),
            owner_acc.clone(),
            pool_pda.clone(),
        )?;
        // transfer fee vault
        if fee_vault.amount > 0 {
            Self::token_transfer_signed(
                pool_acc.clone(),
                pool.nonce,
                token_program_acc.clone(),
                fee_vault_acc.clone(),
                owner_token_b_acc.clone(),
                pool_pda.clone(),
                fee_vault.amount,
            )?;
        }
        Self::token_close_signed(
            pool_acc.clone(),
            pool.nonce,
            token_program_acc.clone(),
            fee_vault_acc.clone(),
            owner_acc.clone(),
            pool_pda.clone(),
        )?;
        // close account
        {
            let user_lamports = owner_acc.lamports();
            **owner_acc.lamports.borrow_mut() =
                user_lamports.checked_add(pool_acc.lamports()).unwrap();
            **pool_acc.lamports.borrow_mut() = 0;
            sol_memset(*pool_acc.data.borrow_mut(), 0, AmmPool::LEN);
        }
        Ok(())
    }

    /// Processes `Swap` instruction.
    fn process_swap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        direction: Direction,
    ) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 9];
        let [pool_acc, vault_a_acc, vault_b_acc, fee_vault_acc, pool_pda, user_wallet_acc, user_token_a_acc, user_token_b_acc, token_program_acc] =
            accounts;
        // use data
        let pool = AmmPool::unpack_unchecked(&pool_acc.data.borrow())?;
        let vault_a = Self::unpack_token_account(vault_a_acc)?;
        let vault_b = Self::unpack_token_account(vault_b_acc)?;
        // check
        if !user_wallet_acc.is_signer {
            msg!("user wallet must sign");
            return Err(AmmError::InvalidSignAccount.into());
        }
        Self::check_account_owner(pool_acc, program_id)?;
        if pool.vault_a != *vault_a_acc.key {
            msg!("vault a not match {} {}", pool.vault_a, *vault_a_acc.key);
            return Err(AmmError::InvalidVault.into());
        }
        if pool.vault_b != *vault_b_acc.key {
            msg!("vault b not match {} {}", pool.vault_b, *vault_b_acc.key);
            return Err(AmmError::InvalidVault.into());
        }
        if pool.status != PoolStatus::Nomal {
            msg!("pool status:{}", pool.status);
            return Err(AmmError::PoolLock.into());
        }
        // check user token
        let user_token_a = Self::unpack_token_account(user_token_a_acc)?;
        let user_token_b = Self::unpack_token_account(user_token_b_acc)?;
        if user_token_a.mint != vault_a.mint {
            msg!(
                "user token a not match {} {}",
                user_token_a.mint,
                vault_a.mint
            );
            return Err(AmmError::InvalidMint.into());
        }
        if user_token_b.mint != vault_b.mint {
            msg!(
                "user token b not match {} {}",
                user_token_b.mint,
                vault_b.mint
            );
            return Err(AmmError::InvalidMint.into());
        }
        // match direction
        let amount_transfer: u64;
        msg!(
            "amount:{}, vault a:{}, vault b:{}",
            amount,
            vault_a.amount,
            vault_b.amount
        );
        match direction {
            Direction::A2B => {
                amount_transfer = Self::calculate_amount_a2b(pool, amount, vault_a, vault_b)?;
                // transfer user token to vault
                Self::token_transfer(
                    token_program_acc.clone(),
                    user_token_a_acc.clone(),
                    vault_a_acc.clone(),
                    user_wallet_acc.clone(),
                    amount,
                )?;
                // transfer vault token to user
                Self::token_transfer_signed(
                    pool_acc.clone(),
                    pool.nonce,
                    token_program_acc.clone(),
                    vault_b_acc.clone(),
                    user_token_b_acc.clone(),
                    pool_pda.clone(),
                    amount_transfer,
                )?;
            }
            Direction::B2A => {
                amount_transfer = Self::calculate_amount_b2a(pool, amount, vault_a, vault_b)?;
                // transfer user token to vault
                Self::token_transfer(
                    token_program_acc.clone(),
                    user_token_b_acc.clone(),
                    vault_b_acc.clone(),
                    user_wallet_acc.clone(),
                    amount_transfer,
                )?;
                // transfer vault token to user
                Self::token_transfer_signed(
                    pool_acc.clone(),
                    pool.nonce,
                    token_program_acc.clone(),
                    vault_a_acc.clone(),
                    user_token_a_acc.clone(),
                    pool_pda.clone(),
                    amount,
                )?;
            }
        }
        // check if k is within tolerance
        Self::check_amount_tolerance(pool, direction, amount, amount_transfer, vault_a, vault_b)?;
        // transfer fee
        let fee_mount = amount_transfer
            .checked_mul(pool.fee)
            .and_then(|v| v.checked_div(PERCENT_MUL))
            .unwrap();
        if fee_mount > 0 {
            // transfer user token to vault
            Self::token_transfer(
                token_program_acc.clone(),
                user_token_b_acc.clone(),
                fee_vault_acc.clone(),
                user_wallet_acc.clone(),
                fee_mount,
            )?;
        }
        Ok(())
    }

    /// Processes `Withdrawal Fee` instruction.
    fn process_withdrawal_fee(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 6];
        let [pool_acc, owner_acc, fee_vault_acc, fee_receiver_acc, pool_pda, token_program_acc] =
            accounts;
        // use data
        let pool = AmmPool::unpack_unchecked(&pool_acc.data.borrow())?;
        // check
        if !owner_acc.is_signer {
            msg!("owner must sign");
            return Err(AmmError::InvalidSignAccount.into());
        }
        Self::check_account_owner(pool_acc, program_id)?;
        if pool.owner != *owner_acc.key {
            msg!("owner not match {} {}", pool.owner, *owner_acc.key);
            return Err(AmmError::InvalidOwner.into());
        }
        // check fee vault
        let fee_vault = Self::unpack_token_account(fee_vault_acc)?;
        if pool.fee_vault != *fee_vault_acc.key {
            msg!(
                "fee vault not match {} {}",
                pool.fee_vault,
                *fee_vault_acc.key
            );
            return Err(AmmError::InvalidVault.into());
        }
        // transfer fee to receiver
        if fee_vault.amount > 0 {
            Self::token_transfer_signed(
                pool_acc.clone(),
                pool.nonce,
                token_program_acc.clone(),
                fee_vault_acc.clone(),
                fee_receiver_acc.clone(),
                pool_pda.clone(),
                fee_vault.amount,
            )?;
        } else {
            return Err(AmmError::NoFee.into());
        }
        Ok(())
    }
    /// calculate a2b amount
    /// A*B=k
    /// (A+a)*(B-b)=k
    /// b=B-k/(A+a)
    fn calculate_amount_a2b(
        pool: AmmPool,
        amount_a: u64,
        vault_a: spl_token::state::Account,
        vault_b: spl_token::state::Account,
    ) -> Result<u64, AmmError> {
        // calcaulate k
        let ka = pool.ka as u128;
        let kb = pool.kb as u128;
        let k: u128 = ka.checked_mul(kb).unwrap();
        // calculate amount a
        let amount = amount_a as u128;
        let vault_a_amount = vault_a.amount as u128;
        let vault_b_amount = vault_b.amount as u128;
        let changed_a = vault_a_amount.checked_add(amount).unwrap();
        let temp: u128 = k.checked_div(changed_a).unwrap();
        let amount_b: u128 = vault_b_amount.checked_sub(temp).unwrap();
        if amount_b >= vault_b_amount {
            msg!(
                "amount b too big, vault:{}, amount:{}",
                vault_b.amount,
                amount_b
            );
            return Err(AmmError::CalculationError);
        }
        if amount_b > u64::MAX as u128 {
            msg!("amount b too big:{}", amount_b);
            return Err(AmmError::CalculationError);
        }
        Ok(amount_b as u64)
    }

    /// calculate b2a amount
    /// A*B=k
    /// (A-a)*(B+b)=k
    /// b=k/(A-a)-B
    fn calculate_amount_b2a(
        pool: AmmPool,
        amount_a: u64,
        vault_a: spl_token::state::Account,
        vault_b: spl_token::state::Account,
    ) -> Result<u64, AmmError> {
        // calculatek
        let ka = pool.ka as u128;
        let kb = pool.kb as u128;
        let k: u128 = ka.checked_mul(kb).unwrap();
        // calculate amount
        let amount = amount_a as u128;
        let vault_a_amount = vault_a.amount as u128;
        let vault_b_amount = vault_b.amount as u128;
        let changed_a: u128 = vault_a_amount.checked_sub(amount).unwrap();
        if changed_a == 0 {
            msg!(
                "amount a too big, vault:{}, amount:{}",
                vault_a.amount,
                amount_a
            );
            return Err(AmmError::CalculationError);
        }
        let temp: u128 = k.checked_div(changed_a).unwrap();
        let amount_b: u128 = temp.checked_sub(vault_b_amount).unwrap();
        if amount_b > u64::MAX as u128 {
            msg!("amount b too big:{}", amount_b);
            return Err(AmmError::CalculationError);
        }
        Ok(amount_b as u64)
    }

    fn check_amount_tolerance(
        pool: AmmPool,
        direction: Direction,
        amount: u64,
        amount_transfer: u64,
        vault_a: spl_token::state::Account,
        vault_b: spl_token::state::Account,
    ) -> Result<(), AmmError> {
        // calculate k
        let ka = pool.ka as u128;
        let kb = pool.kb as u128;
        let k_origin: u128 = ka.checked_mul(kb).unwrap();
        let ka_new: u128;
        let kb_new: u128;
        // amount
        let vault_a_amount = vault_a.amount as u128;
        let vault_b_amount = vault_b.amount as u128;
        let amount_big = amount as u128;
        let amount_transfer_big = amount_transfer as u128;
        match direction {
            Direction::A2B => {
                ka_new = vault_a_amount.checked_add(amount_big).unwrap();
                kb_new = vault_b_amount.checked_sub(amount_transfer_big).unwrap();
            }
            Direction::B2A => {
                ka_new = vault_a_amount.checked_sub(amount_big).unwrap();
                kb_new = vault_b_amount.checked_add(amount_transfer_big).unwrap();
            }
        }
        let k_new: u128 = ka_new.checked_mul(kb_new).unwrap();
        let tolerance: u128;
        if k_origin > k_new {
            tolerance = k_origin - k_new;
        } else if k_new > k_origin {
            tolerance = k_new - k_origin;
        } else {
            tolerance = 0;
        }
        if tolerance > pool.tolerance as u128 {
            msg!(
                "tolerance too big, pool tolerance:{}, calculated tolerance:{}",
                pool.tolerance,
                tolerance
            );
            return Err(AmmError::OutOfTolerance);
        }
        Ok(())
    }

    /// Check account owner is the given program
    fn check_account_owner(
        account_info: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<(), AmmError> {
        if *program_id != *account_info.owner {
            msg!(
                "Expected account to be owned by program {}, received {}",
                program_id,
                account_info.owner
            );
            Err(AmmError::InvalidProgramAddress)
        } else {
            Ok(())
        }
    }

    /// Unpacks a spl_token `Account`.
    fn unpack_token_account(
        account_info: &AccountInfo,
    ) -> Result<spl_token::state::Account, AmmError> {
        if account_info.owner != &spl_token::ID {
            Err(AmmError::InvalidTokenProgramId)
        } else {
            spl_token::state::Account::unpack(&account_info.data.borrow())
                .map_err(|_| AmmError::ExpectedAccount)
        }
    }

    /// Unpacks a spl_token `Mint`.
    fn _unpack_mint(account_info: &AccountInfo) -> Result<spl_token::state::Mint, AmmError> {
        if account_info.owner != &spl_token::ID {
            Err(AmmError::InvalidTokenProgramId)
        } else {
            spl_token::state::Mint::unpack(&account_info.data.borrow())
                .map_err(|_| AmmError::ExpectedMint)
        }
    }

    /// Issue a spl_token `Transfer` instruction.
    fn token_transfer<'a>(
        token_program: AccountInfo<'a>,
        source: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        amount: u64,
    ) -> Result<(), ProgramError> {
        if amount == 0 {
            msg!("transfer amount is zero.");
            return Err(ProgramError::Custom(321));
        }
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

    /// Issue a spl_token `Transfer` instruction by pda.
    fn token_transfer_signed<'a>(
        pool_acc: AccountInfo<'a>,
        nonce: u8,
        token_program_acc: AccountInfo<'a>,
        source_acc: AccountInfo<'a>,
        destination_acc: AccountInfo<'a>,
        pda: AccountInfo<'a>,
        amount: u64,
    ) -> Result<(), ProgramError> {
        if amount == 0 {
            msg!("transfer amount is zero.");
            return Err(ProgramError::Custom(321));
        }
        let seeds = &[pool_acc.key.as_ref(), &[nonce]];
        let signers = &[&seeds[..]];
        let ix = spl_token::instruction::transfer(
            token_program_acc.key,
            source_acc.key,
            destination_acc.key,
            pda.key,
            &[&pda.key],
            amount,
        )?;
        invoke_signed(
            &ix,
            &[source_acc, destination_acc, pda, token_program_acc],
            signers,
        )
    }

    /// Issue a spl_token `Close` instruction by pda.
    fn token_close_signed<'a>(
        pool_acc: AccountInfo<'a>,
        nonce: u8,
        token_program_acc: AccountInfo<'a>,
        account_acc: AccountInfo<'a>,
        destination_acc: AccountInfo<'a>,
        pda: AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let seeds = &[pool_acc.key.as_ref(), &[nonce]];
        let signers = &[&seeds[..]];
        let ix = spl_token::instruction::close_account(
            token_program_acc.key,
            account_acc.key,
            destination_acc.key,
            pda.key,
            &[&pda.key],
        )?;
        invoke_signed(
            &ix,
            &[account_acc, destination_acc, pda, token_program_acc],
            signers,
        )
    }
}

impl PrintProgramError for AmmError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            AmmError::InvalidProgramAddress => msg!("Error: InvalidProgramAddress"),
            AmmError::InvalidOwner => msg!("Error: InvalidOwner"),
            AmmError::ExpectedMint => msg!("Error: ExpectedMint"),
            AmmError::ExpectedAccount => msg!("Error: ExpectedAccount"),
            AmmError::InvalidTokenProgramId => msg!("Error: InvalidTokenProgramId"),
            AmmError::InvalidInstruction => msg!("Error: InvalidInstruction"),
            AmmError::InvalidSignAccount => msg!("Error: InvalidSignAccount"),
            AmmError::InvalidVault => msg!("Error: InvalidVault"),
            AmmError::InvalidMint => msg!("Error: InvalidMint"),
            AmmError::InvalidStatus => msg!("Error: InvalidStatus"),
            AmmError::InsufficientFunds => msg!("Error: InsufficientFunds"),
            AmmError::InvalidInput => msg!("Error: InvalidInput"),
            AmmError::PoolExist => msg!("Error: PoolExist"),
            AmmError::PoolLock => msg!("Error: PoolLock"),
            AmmError::InvalidAmount => msg!("Error: Amount must be greater than zero."),
            AmmError::NoFee => msg!("Error: NoFee"),
            AmmError::InvalidDirection => msg!("Error: InvalidDirection"),
            AmmError::CalculationError => msg!("Error: CalculationError"),
            AmmError::OutOfTolerance => msg!("Error: OutOfTolerance"),
            AmmError::NoughtTransfer => msg!("Error: NoughtTransfer"),
            AmmError::InvalidPDA => msg!("Error: InvalidPDA"),
        }
    }
}

// public key of 11111111111111111111111111111111
const _NULL_PUBKEY: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);
// public key of usdc
const _USDC_PUBKEY: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array([
        198, 250, 122, 243, 190, 219, 173, 58, 61, 101, 243, 106, 171, 201, 116, 49, 177, 187, 228,
        194, 210, 246, 224, 228, 124, 166, 2, 3, 69, 47, 93, 97,
    ]);

const PERCENT_MUL: u64 = u64::pow(10, 6);
