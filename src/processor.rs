//! Program state processor
// use solana_program::sysvar::Sysvar;
use {
    crate::{error::AmmError, instruction::SapInstruction, state::AmmPool},
    arrayref::array_ref,
    num_traits::FromPrimitive,
    // pyth_client::{CorpAction, PriceStatus, PriceType},
    solana_program::{
        account_info::AccountInfo,
        // clock::Clock,
        decode_error::DecodeError,
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::{PrintProgramError, ProgramError},
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
                amount_a,
                amount_b,
                tolerance,
            } => {
                msg!("Instruction: Init");
                Self::process_initialize(
                    program_id, accounts, nonce, fee_1, fee_2, fee_3, fee_4, fee_5, amount_a,
                    amount_b, tolerance,
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
            SapInstruction::UpdateTolerance { tolerance } => {
                msg!("Instruction: Update Tolerance");
                Self::process_update_tolerance(program_id, accounts, tolerance)
            }
            SapInstruction::Swap { amount, direction } => {
                msg!("Instruction: Swap");
                Self::process_swap(program_id, accounts, amount, direction)
            }
        }
    }

    /// Processes `Initialize` instruction.
    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        nonce: u8,
        fee_1: u64,
        fee_2: u64,
        fee_3: u64,
        fee_4: u64,
        fee_5: u64,
        amount_a: u64,
        amount_b: u64,
        tolerance: u64,
    ) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 17];
        let [pool_acc, owner_acc, mint_a_acc, mint_b_acc, vault_a_acc, vault_b_acc, fee_vault_acc, fee_receiver_1_acc, fee_receiver_2_acc, fee_receiver_3_acc, fee_receiver_4_acc, fee_receiver_5_acc, fee_mint_acc, pool_pda, owner_token_a_acc, owner_token_b_acc, token_program_acc] =
            accounts;
        // use data
        let mut pool = AmmPool::unpack(&pool_acc.data.borrow())?;
        // check
        if !owner_acc.is_signer {
            msg!("owner must sign");
            return Err(AmmError::InvalidSignAccount.into());
        }
        Self::check_account_owner(pool_acc, program_id)?;
        if pool.status != 0 {
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
        if fee_vault.mint != *fee_mint_acc.key {
            msg!(
                "fee vault mint not match {} {}",
                fee_vault.mint,
                *mint_a_acc.key
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
        pool.status = 1;
        pool.nonce = nonce;
        pool.ka = amount_a;
        pool.kb = amount_b;
        pool.tolerance = tolerance;
        pool.fee_1 = fee_1;
        pool.fee_2 = fee_2;
        pool.fee_3 = fee_3;
        pool.fee_4 = fee_4;
        pool.fee_5 = fee_5;
        pool.owner = *owner_acc.key;
        pool.mint_a = *mint_a_acc.key;
        pool.mint_b = *mint_b_acc.key;
        pool.vault_a = *vault_a_acc.key;
        pool.vault_b = *vault_b_acc.key;
        pool.fee_vault = *fee_vault_acc.key;
        pool.fee_receiver_1 = *fee_receiver_1_acc.key;
        pool.fee_receiver_2 = *fee_receiver_2_acc.key;
        pool.fee_receiver_3 = *fee_receiver_3_acc.key;
        pool.fee_receiver_4 = *fee_receiver_4_acc.key;
        pool.fee_receiver_5 = *fee_receiver_5_acc.key;
        pool.fee_mint = *fee_mint_acc.key;
        // pack pool
        AmmPool::pack(pool, &mut pool_acc.data.borrow_mut())?;
        Ok(())
    }

    /// Processes `Update Pool` instruction.
    fn process_update_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 7];
        let [pool_acc, owner_acc, fee_receiver_1_acc, fee_receiver_2_acc, fee_receiver_3_acc, fee_receiver_4_acc, fee_receiver_5_acc] =
            accounts;
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
        if pool.status == 0 {
            msg!("pool status:{}", pool.status);
            return Err(AmmError::InvalidStatus.into());
        }
        // update pool
        pool.fee_receiver_1 = *fee_receiver_1_acc.key;
        pool.fee_receiver_2 = *fee_receiver_2_acc.key;
        pool.fee_receiver_3 = *fee_receiver_3_acc.key;
        pool.fee_receiver_4 = *fee_receiver_4_acc.key;
        pool.fee_receiver_5 = *fee_receiver_5_acc.key;
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
        if pool.status == 0 {
            msg!("pool status:{}", pool.status);
            return Err(AmmError::InvalidStatus.into());
        }
        if status < 1 || status > 2 {
            msg!("status:{}", status);
            return Err(AmmError::InvalidStatus.into());
        }
        // update pool
        pool.status = status;
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
        if pool.status == 0 {
            msg!("pool status:{}", pool.status);
            return Err(AmmError::InvalidStatus.into());
        }
        // update pool
        pool.tolerance = tolerance;
        // pack pool
        AmmPool::pack(pool, &mut pool_acc.data.borrow_mut())?;
        Ok(())
    }

    /// Processes `Swap` instruction.
    fn process_swap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        direction: u8,
    ) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 10];
        let [pool_acc, vault_a_acc, vault_b_acc, _fee_vault, pool_pda, user_wallet_acc, user_token_a_acc, user_token_b_acc, _user_fee_acc, token_program_acc] =
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
        if pool.status != 1 {
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
        // 1 is a2b, 2 is b2a
        let amount_transfer: u64;
        match direction {
            1 => {
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
            2 => {
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
            _ => {
                return Err(AmmError::InvalidMint.into());
            }
        }
        // check if k is within tolerance
        Self::check_amount_tolerance(pool, amount, amount_transfer)?;
        Ok(())
    }

    /// calculate a2b amount
    /// A*B=k
    /// (A-a)*(B+b)=k
    /// b=k/(A-a)-B
    fn calculate_amount_a2b(
        pool: AmmPool,
        amount_a: u64,
        vault_a: spl_token::state::Account,
        vault_b: spl_token::state::Account,
    ) -> Result<u64, AmmError> {
        let k: u64 = pool.ka.checked_mul(pool.kb).unwrap();
        let changed_a = vault_a.amount.checked_sub(amount_a).unwrap();
        if changed_a == 0 {
            msg!(
                "amount a too big, vault:{}, amount:{}",
                vault_a.amount,
                amount_a
            );
            return Err(AmmError::CalculationError);
        }
        let amount_b = k
            .checked_div(changed_a)
            .and_then(|v| v.checked_sub(vault_b.amount))
            .unwrap();
        Ok(amount_b)
    }

    /// calculate b2a amount
    /// A*B=k
    /// (A+a)*(B-b)=k
    /// b=B-k/(A+a)
    fn calculate_amount_b2a(
        pool: AmmPool,
        amount_a: u64,
        vault_a: spl_token::state::Account,
        vault_b: spl_token::state::Account,
    ) -> Result<u64, AmmError> {
        let k: u64 = pool.ka.checked_mul(pool.kb).unwrap();
        let changed_a = vault_a.amount.checked_add(amount_a).unwrap();
        let temp: u64 = k.checked_div(changed_a).unwrap();
        let amount_b: u64 = vault_b.amount.checked_sub(temp).unwrap();
        if amount_b >= vault_b.amount {
            msg!(
                "amount b too big, vault:{}, amount:{}",
                vault_b.amount,
                amount_b
            );
            return Err(AmmError::CalculationError);
        }
        Ok(amount_b)
    }

    fn check_amount_tolerance(
        pool: AmmPool,
        amount: u64,
        amount_transfer: u64,
    ) -> Result<(), AmmError> {
        let k_origin: u64 = pool.ka.checked_mul(pool.kb).unwrap();
        let k_new: u64 = amount.checked_mul(amount_transfer).unwrap();
        let tolerance: u64;
        if k_origin > k_new {
            tolerance = k_origin - k_new;
        } else if k_new > k_origin {
            tolerance = k_new - k_origin;
        } else {
            tolerance = 0;
        }
        if tolerance > pool.tolerance {
            msg!(
                "tolerance too big, pool:{}, calculation:{}",
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

const _PERCENT_MUL: u64 = u64::pow(10, 6);
