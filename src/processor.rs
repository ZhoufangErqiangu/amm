//! Program state processor
// use solana_program::sysvar::Sysvar;
use {
    crate::{error::AmmError, instruction::SapInstruction, state::AmmPool},
    arrayref::array_ref,
    // std::convert::TryInto,
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
            } => {
                msg!("Instruction: Init");
                Self::process_initialize(
                    program_id, accounts, nonce, fee_1, fee_2, fee_3, fee_4, fee_5, amount_a,
                    amount_b,
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
        fee_1: f64,
        fee_2: f64,
        fee_3: f64,
        fee_4: f64,
        fee_5: f64,
        amount_a: u64,
        amount_b: u64,
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
        Self::check_account_owner(pool_acc, program_id);
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
        );
        Self::token_transfer(
            token_program_acc.clone(),
            owner_token_b_acc.clone(),
            vault_b_acc.clone(),
            owner_acc.clone(),
            amount_b,
        );
        // init pool
        pool.status = 1;
        pool.nonce = nonce;
        pool.ka = amount_a;
        pool.kb = amount_b;
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
        Self::check_account_owner(pool_acc, program_id);
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
        Self::check_account_owner(pool_acc, program_id);
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

    /// Processes `Swap` instruction.
    fn process_swap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        direction: u8,
    ) -> ProgramResult {
        let accounts = array_ref![accounts, 0, 11];
        let [pool_acc, mint_a_acc, mint_b_acc, vault_a_acc, vault_b_acc, fee_vault, fee_mint, user_wallet_acc, user_token_a_acc, user_token_b_acc, token_program_acc] =
            accounts;
        // use data
        let pool = AmmPool::unpack_unchecked(&pool_acc.data.borrow())?;
        // check
        if !user_wallet_acc.is_signer {
            msg!("user wallet must sign");
            return Err(AmmError::InvalidSignAccount.into());
        }
        Self::check_account_owner(pool_acc, program_id);
        if pool.mint_a != *mint_a_acc.key {
            msg!("mint a not match {} {}", pool.mint_a, *mint_a_acc.key);
            return Err(AmmError::InvalidMint.into());
        }
        if pool.mint_b != *mint_b_acc.key {
            msg!("mint b not match {} {}", pool.mint_b, *mint_b_acc.key);
            return Err(AmmError::InvalidMint.into());
        }
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
        if user_token_a.mint != *mint_a_acc.key {
            msg!(
                "user token a not match {} {}",
                user_token_a.mint,
                *mint_a_acc.key
            );
            return Err(AmmError::InvalidMint.into());
        }
        if user_token_b.mint != *mint_b_acc.key {
            msg!(
                "user token b not match {} {}",
                user_token_b.mint,
                *mint_b_acc.key
            );
            return Err(AmmError::InvalidMint.into());
        }
        // match direction
        match direction {
            1 => {}
            2 => {}
            _ => {
                return Err(AmmError::InvalidMint.into());
            }
        }
        Ok(())
    }

    /// calculate a2b amount
    fn calculate_amount_a2b(pool: AmmPool, amount: u64) -> Result<u64, AmmError> {
        Ok(amount)
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
    fn unpack_mint(account_info: &AccountInfo) -> Result<spl_token::state::Mint, AmmError> {
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
        // generate signer seeds
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
