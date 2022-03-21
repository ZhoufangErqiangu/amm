//! Instruction types
#![allow(clippy::too_many_arguments)]

use crate::error::SapError;
use crate::state::TOKEN_NUM;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
use std::convert::TryInto;
use std::mem::size_of;

use arrayref::{array_ref, array_refs};

/// Instructions supported by the token program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]

pub enum SapInstruction {
    Initialize {
        nonce: u64,
        sap_init_price: f64,
        time_lock: i64,
        sap_pre_mint_l1_amount: u64,
        sap_pre_mint_l2_amount: u64,
    },
    Mint {
        token_pubkey: Pubkey,
        amount_in: u64,
        ltype: u64,
        fee: u64,
    },
    Redeem {
        amount_in: u64,
        ltype: u64,
        burn_fee: u64,
        sap_cost: u64,
        performance_fee: u64,
    },
    Trade {
        token_to_trade_pubkey: Pubkey,
        token_amount: u64,
        ask_price: u64,
        bid_price: u64,
        trade_type: u64,
        token_to_buy_pubkey: Pubkey,
    },
    UpdateTokenList {},
    UpdateManager {
        manager_pubkey: Pubkey,
    },
    WithdrawalFee {},
    CreateSapMember {
        nonce: u64,
        ltype: u64,
    },
    UpdateSap {
        status: u64,
        sap_pre_mint_l1_amount: u64,
        sap_pre_mint_l2_amount: u64,
    },
    ClaimSap {},
    StartSap {},
    PreMintSap {
        amount: u64,
        fee: u64,
    },
    PreBurnSap {
        amount: u64,
        fee: u64,
    },
    UpdateStatus {
        status: u64,
    },
}

impl SapInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(crate::error::SapError::InvalidInstruction)?;
        // msg!("{:?} {:?} xxxx",tag,rest);
        Ok(match tag {
            // initialize
            0 => {
                let data = array_ref![rest, 0, 8 * 5];
                let (
                    nonce_buf,
                    sap_init_price_buf,
                    time_lock_buf,
                    sap_pre_mint_l1_amount_buf,
                    sap_pre_mint_l2_amount_buf,
                ) = array_refs![data, 8, 8, 8, 8, 8];

                Self::Initialize {
                    nonce: u64::from_le_bytes(*nonce_buf),
                    sap_init_price: f64::from_le_bytes(*sap_init_price_buf),
                    time_lock: i64::from_le_bytes(*time_lock_buf),
                    sap_pre_mint_l1_amount: u64::from_le_bytes(*sap_pre_mint_l1_amount_buf),
                    sap_pre_mint_l2_amount: u64::from_le_bytes(*sap_pre_mint_l2_amount_buf),
                }
            }

            // user buys token and sap mint corresponding token
            1 => {
                let data = array_ref![rest, 0, 32 + 8 + 8 + 8];
                let (token_pubkey, amount_in, ltype, fee) = array_refs![data, 32, 8, 8, 8];

                Self::Mint {
                    token_pubkey: Self::unpack_pubkey(token_pubkey)?,
                    amount_in: u64::from_le_bytes(*amount_in),
                    ltype: u64::from_le_bytes(*ltype),
                    fee: u64::from_le_bytes(*fee),
                }
            }

            // user redeem sap token
            2 => {
                let data = array_ref![rest, 0, 8 * 5];
                let (amount_in, ltype, burn_fee, sap_cost, performance_fee) =
                    array_refs![data, 8, 8, 8, 8, 8];
                Self::Redeem {
                    amount_in: u64::from_le_bytes(*amount_in),
                    ltype: u64::from_le_bytes(*ltype),
                    burn_fee: u64::from_le_bytes(*burn_fee),
                    sap_cost: u64::from_le_bytes(*sap_cost),
                    performance_fee: u64::from_le_bytes(*performance_fee),
                }
            }

            // manager trade
            3 => {
                let data = array_ref![rest, 0, 32 + 8 + 8 + 8 + 8 + 32];
                let (
                    token_to_trade_pubkey,
                    token_amount,
                    ask_price,
                    bid_price,
                    trade_type,
                    token_to_buy_pubkey,
                ) = array_refs![data, 32, 8, 8, 8, 8, 32];

                Self::Trade {
                    token_to_trade_pubkey: Self::unpack_pubkey(token_to_trade_pubkey)?,
                    token_amount: u64::from_le_bytes(*token_amount),
                    ask_price: u64::from_le_bytes(*ask_price),
                    bid_price: u64::from_le_bytes(*bid_price),
                    trade_type: u64::from_le_bytes(*trade_type),
                    token_to_buy_pubkey: Self::unpack_pubkey(token_to_buy_pubkey)?,
                }
            }

            // update sap token list
            4 => Self::UpdateTokenList {},

            // update sap manager pubkey
            5 => Self::UpdateManager {
                manager_pubkey: Self::unpack_pubkey(rest)?,
            },

            // 6 =>{
            //     Self::Update_Oracle{
            //         oracle_pubkey : Self::unpack_pubkey(rest)?
            //     }
            // }
            7 => {
                let data = array_ref![rest, 0, 8 + 8];
                let (nonce, ltype) = array_refs![data, 8, 8];
                Self::CreateSapMember {
                    nonce: u64::from_le_bytes(*nonce),
                    ltype: u64::from_le_bytes(*ltype),
                }
            }

            8 => {
                let data = array_ref![rest, 0, 8 + 8 + 8];
                let (status, sap_pre_mint_l1_amount, sap_pre_mint_l2_amount) =
                    array_refs![data, 8, 8, 8];
                Self::UpdateSap {
                    status: u64::from_le_bytes(*status),
                    sap_pre_mint_l1_amount: u64::from_le_bytes(*sap_pre_mint_l1_amount),
                    sap_pre_mint_l2_amount: u64::from_le_bytes(*sap_pre_mint_l2_amount),
                }
            }

            9 => Self::ClaimSap {},

            10 => Self::StartSap {},

            11 => {
                let data = array_ref![rest, 0, 8 + 8];
                let (amount, fee) = array_refs![data, 8, 8];
                Self::PreMintSap {
                    amount: u64::from_le_bytes(*amount),
                    fee: u64::from_le_bytes(*fee),
                }
            }

            12 => {
                let data = array_ref![rest, 0, 8 + 8];
                let (amount, fee) = array_refs![data, 8, 8];
                Self::PreBurnSap {
                    amount: u64::from_le_bytes(*amount),
                    fee: u64::from_le_bytes(*fee),
                }
            }

            13 => {
                let data = array_ref![rest, 0, 8];
                Self::UpdateStatus {
                    status: u64::from_le_bytes(*data),
                }
            }

            21 => Self::WithdrawalFee {},

            _ => return Err(SapError::InvalidInstruction.into()),
        })
    }

    // unpack byte array to u64
    fn _unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(SapError::InvalidInstruction)?;
        Ok(amount)
    }

    fn unpack_pubkey(input: &[u8]) -> Result<Pubkey, ProgramError> {
        if input.len() >= 32 {
            let (key, _rest) = input.split_at(32);
            let pk = Pubkey::new(key);
            Ok(pk)
        } else {
            Err(SapError::InvalidInstruction.into())
        }
    }

    // pack function to pack a SapInstruction enum into a byte array for test convenience
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            // pack initialize
            &Self::Initialize {
                nonce,
                sap_init_price,
                time_lock,
                sap_pre_mint_l1_amount,
                sap_pre_mint_l2_amount,
            } => {
                buf.push(0);
                buf.extend_from_slice(&nonce.to_le_bytes());
                buf.extend_from_slice(&sap_init_price.to_le_bytes());
                buf.extend_from_slice(&time_lock.to_le_bytes());
                buf.extend_from_slice(&sap_pre_mint_l1_amount.to_le_bytes());
                buf.extend_from_slice(&sap_pre_mint_l2_amount.to_le_bytes());
            }

            // pack mint
            &Self::Mint {
                ref token_pubkey,
                amount_in,
                ltype,
                fee,
            } => {
                buf.push(1);
                buf.extend_from_slice(token_pubkey.as_ref());
                buf.extend_from_slice(&amount_in.to_le_bytes());
                buf.extend_from_slice(&ltype.to_le_bytes());
                buf.extend_from_slice(&fee.to_le_bytes());
            }

            // pack redeem
            &Self::Redeem {
                amount_in,
                ltype,
                burn_fee,
                sap_cost,
                performance_fee,
            } => {
                buf.push(2);
                // buf.extend_from_slice(token_pubkey.as_ref());
                buf.extend_from_slice(&amount_in.to_le_bytes());
                buf.extend_from_slice(&ltype.to_le_bytes());
                buf.extend_from_slice(&burn_fee.to_le_bytes());
                buf.extend_from_slice(&sap_cost.to_le_bytes());
                buf.extend_from_slice(&performance_fee.to_le_bytes());
            }

            // pack trade
            &Self::Trade {
                ref token_to_trade_pubkey,
                token_amount,
                ask_price,
                bid_price,
                trade_type,
                ref token_to_buy_pubkey,
            } => {
                buf.push(3);
                buf.extend_from_slice(token_to_trade_pubkey.as_ref());
                buf.extend_from_slice(&token_amount.to_le_bytes());
                buf.extend_from_slice(&ask_price.to_le_bytes());
                buf.extend_from_slice(&bid_price.to_le_bytes());
                buf.extend_from_slice(&trade_type.to_le_bytes());
                buf.extend_from_slice(token_to_buy_pubkey.as_ref());
            }

            // pack update token prices
            &Self::UpdateTokenList {} => {
                buf.push(4);
                // for i in 0..TOKEN_NUM {
                //     buf.extend_from_slice(token_list[i].as_ref());
                // }
                // for i in 0..TOKEN_NUM {
                //     buf.extend_from_slice(token_assets_vault_pubkeys[i].as_ref());
                // }
            }

            // pack update token composition percentage
            &Self::UpdateManager { manager_pubkey } => {
                buf.push(5);
                buf.extend_from_slice(manager_pubkey.as_ref());
            }

            // update oracle pubkey
            // &Self::Update_Oracle {
            //     oracle_pubkey,
            // } => {
            //         buf.push(6);
            //         buf.extend_from_slice(oracle_pubkey.as_ref());
            //     }
            // }

            // pack create member
            &Self::CreateSapMember { nonce, ltype } => {
                buf.push(7);
                buf.extend_from_slice(&nonce.to_le_bytes());
                buf.extend_from_slice(&ltype.to_le_bytes());
            }

            // pack update sap
            &Self::UpdateSap {
                status,
                sap_pre_mint_l1_amount,
                sap_pre_mint_l2_amount,
            } => {
                buf.push(8);
                buf.extend_from_slice(&status.to_le_bytes());
                buf.extend_from_slice(&sap_pre_mint_l1_amount.to_le_bytes());
                buf.extend_from_slice(&sap_pre_mint_l2_amount.to_le_bytes());
            }

            // pack claim sap
            &Self::ClaimSap {} => {
                buf.push(9);
            }

            // pack start sap
            &Self::StartSap {} => {
                buf.push(10);
            }

            &Self::PreMintSap { amount, fee } => {
                buf.push(11);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&fee.to_le_bytes());
            }

            &Self::PreBurnSap { amount, fee } => {
                buf.push(12);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&fee.to_le_bytes());
            }

            &Self::UpdateStatus { status } => {
                buf.push(13);
                buf.extend_from_slice(&status.to_le_bytes());
            }

            &Self::WithdrawalFee {} => {
                buf.push(21);
            }
        }
        buf
    }
}

// create Initialize  instruction
pub fn initialize(
    sap_program_id: &Pubkey,
    sap_account_pubkey: &Pubkey,
    sap_mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    manager_pubkey: &Pubkey,
    // oracle_pubkey : &Pubkey,
    user_sap_pubkey: &Pubkey,
    user_wallet_pubkey: &Pubkey,
    token_assets_vault_pubkeys: [Pubkey; TOKEN_NUM],
    _token_list: [Pubkey; TOKEN_NUM],
    oracle_product_pubkey_list: [Pubkey; TOKEN_NUM],
    oracle_price_pubkey_list: [Pubkey; TOKEN_NUM],
    nonce: u64,
    sap_init_price: f64,
    time_lock: i64,
    sap_pre_mint_l1_amount: u64,
    sap_pre_mint_l2_amount: u64,
) -> Result<Instruction, ProgramError> {
    let data = SapInstruction::Initialize {
        nonce,
        sap_init_price,
        time_lock,
        sap_pre_mint_l1_amount,
        sap_pre_mint_l2_amount,
        // manager_pubkey:*manager_pubkey ,
        // oracle_pubkey: *oracle_pubkey ,
    }
    .pack();

    let mut accounts = vec![
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(*sap_account_pubkey, false),
        AccountMeta::new(*sap_mint_pubkey, false),
        AccountMeta::new_readonly(*mint_authority_pubkey, false),
        AccountMeta::new_readonly(*manager_pubkey, true),
        AccountMeta::new(*user_sap_pubkey, false),
        AccountMeta::new_readonly(*user_wallet_pubkey, true),
        AccountMeta::new_readonly(sysvar::clock::id(), false), // clock account for timing related functions
    ];

    accounts.extend(
        token_assets_vault_pubkeys
            .iter()
            .map(|pk| AccountMeta::new(*pk, false)),
    );
    accounts.extend(
        oracle_product_pubkey_list
            .iter()
            .map(|pk| AccountMeta::new(*pk, false)),
    );
    accounts.extend(
        oracle_price_pubkey_list
            .iter()
            .map(|pk| AccountMeta::new(*pk, false)),
    );

    Ok(Instruction {
        program_id: *sap_program_id,
        accounts,
        data,
    })
}

//create mint instruction
pub fn mint(
    sap_program_id: &Pubkey,
    sap_account_pubkey: &Pubkey,
    sap_mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    user_sap_pubkey: &Pubkey,
    user_wallet_pubkey: &Pubkey,
    _token_assets_vault_pubkeys: &[Pubkey],
    token_pubkey: &Pubkey,
    amount_in: u64,
    ltype: u64,
    fee: u64,
) -> Result<Instruction, ProgramError> {
    let data = SapInstruction::Mint {
        token_pubkey: *token_pubkey,
        amount_in,
        ltype,
        fee,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(*sap_account_pubkey, false),
        AccountMeta::new(*sap_mint_pubkey, false),
        AccountMeta::new_readonly(*mint_authority_pubkey, false),
        AccountMeta::new(*user_sap_pubkey, false),
        AccountMeta::new_readonly(*user_wallet_pubkey, true),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
    ];
    // accounts.extend(new_token_list.iter().map(|pk| AccountMeta::new(*pk, false)));
    Ok(Instruction {
        program_id: *sap_program_id,
        accounts,
        data,
    })
}

//create redeem instruction
pub fn redeem(
    sap_program_id: &Pubkey,
    sap_account_pubkey: &Pubkey,
    sap_mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    user_sap_pubkey: &Pubkey,
    user_wallet_pubkey: &Pubkey,
    _token_assets_vault_pubkeys: &[Pubkey],
    // token_pubkey : &Pubkey,
    amount_in: u64,
    ltype: u64,
    burn_fee: u64,
    sap_cost: u64,
    performance_fee: u64,
) -> Result<Instruction, ProgramError> {
    let data = SapInstruction::Redeem {
        // token_pubkey:*token_pubkey,
        amount_in,
        ltype,
        burn_fee,
        sap_cost,
        performance_fee,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(*sap_account_pubkey, false),
        AccountMeta::new(*sap_mint_pubkey, false),
        AccountMeta::new_readonly(*mint_authority_pubkey, false),
        AccountMeta::new(*user_sap_pubkey, false),
        AccountMeta::new_readonly(*user_wallet_pubkey, true),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
    ];

    // accounts.extend(new_token_list.iter().map(|pk| AccountMeta::new(*pk, false)));

    Ok(Instruction {
        program_id: *sap_program_id,
        accounts,
        data,
    })
}

//create trade instruction

pub fn trade(
    sap_program_id: &Pubkey,
    sap_account_pubkey: &Pubkey,
    authority_pubkey: &Pubkey, // for vault accounts
    manager_pubkey: &Pubkey,
    dex_pubkey: &Pubkey, // dex pubkey
    _token_assets_vault_pubkeys: &[&Pubkey],
    token_to_trade_pubkey: &Pubkey,
    token_to_buy_pubkey: &Pubkey,
    token_to_trade_vault_pubkey: &Pubkey,
    token_to_buy_vault_pubkey: &Pubkey,
    token_amount: u64,
    ask_price: u64,
    bid_price: u64,
    trade_type: u64,
) -> Result<Instruction, ProgramError> {
    let data = SapInstruction::Trade {
        token_to_trade_pubkey: *token_to_trade_pubkey,
        token_amount,
        ask_price,
        bid_price,
        trade_type,
        token_to_buy_pubkey: *token_to_buy_pubkey,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(*sap_account_pubkey, false),
        // AccountMeta::new(*sap_mint_pubkey, false),
        AccountMeta::new_readonly(*authority_pubkey, false),
        AccountMeta::new(*manager_pubkey, true),
        AccountMeta::new(*dex_pubkey, false),
        AccountMeta::new(*token_to_trade_pubkey, false),
        AccountMeta::new(*token_to_buy_pubkey, false),
        AccountMeta::new(*token_to_trade_vault_pubkey, false),
        AccountMeta::new(*token_to_buy_vault_pubkey, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
    ];

    // accounts.extend(new_token_list.iter().map(|pk| AccountMeta::new(*pk, false)));
    Ok(Instruction {
        program_id: *sap_program_id,
        accounts,
        data,
    })
}

// Creates a 'UpdateManager' instruction.
pub fn update_manager(
    program_id: &Pubkey,
    sap_account_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    manager_pubkey: &Pubkey,
    new_manager_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let data = SapInstruction::UpdateManager {
        manager_pubkey: *new_manager_pubkey,
    }
    .pack();
    let accounts = vec![
        AccountMeta::new(*sap_account_pubkey, false),
        AccountMeta::new_readonly(*authority_pubkey, true),
        AccountMeta::new_readonly(*manager_pubkey, false),
        AccountMeta::new_readonly(*new_manager_pubkey, false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
