//! Instruction types
#![allow(clippy::too_many_arguments)]

use crate::error::AmmError;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use std::mem::size_of;

use arrayref::{array_ref, array_refs};

/// Instructions supported by the token program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]

pub enum SapInstruction {
    Initialize {
        nonce: u8,
        fee_1: f64,
        fee_2: f64,
        fee_3: f64,
        fee_4: f64,
        fee_5: f64,
        amount_a: u64,
        amount_b: u64,
    },
    UpdatePool {},
    UpdateStatus {
        status: u8,
    },
    Swap {
        amount: u64,
        direction: u8,
    },
}

impl SapInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(crate::error::AmmError::InvalidInstruction)?;
        // msg!("{:?} {:?} xxxx",tag,rest);
        Ok(match tag {
            // initialize
            0 => {
                let data = array_ref![rest, 0, 1 + 8 * 7];
                let (
                    nonce_buf,
                    fee_1_buf,
                    fee_2_buf,
                    fee_3_buf,
                    fee_4_buf,
                    fee_5_buf,
                    amount_a_buf,
                    amount_b_buf,
                ) = array_refs![data, 1, 8, 8, 8, 8, 8, 8, 8];

                Self::Initialize {
                    nonce: u8::from_le_bytes(*nonce_buf),
                    fee_1: f64::from_le_bytes(*fee_1_buf),
                    fee_2: f64::from_le_bytes(*fee_2_buf),
                    fee_3: f64::from_le_bytes(*fee_3_buf),
                    fee_4: f64::from_le_bytes(*fee_4_buf),
                    fee_5: f64::from_le_bytes(*fee_5_buf),
                    amount_a: u64::from_le_bytes(*amount_a_buf),
                    amount_b: u64::from_le_bytes(*amount_b_buf),
                }
            }

            // update pool
            1 => Self::UpdatePool {},

            // update pool status
            2 => {
                let data = array_ref![rest, 0, 1];
                Self::UpdateStatus {
                    status: u8::from_le_bytes(*data),
                }
            }

            // swap
            10 => {
                let data = array_ref![rest, 0, 8 + 1];
                let (amount_buf, direction_buf) = array_refs![data, 8, 1];

                Self::Swap {
                    amount: u64::from_le_bytes(*amount_buf),
                    // 1 is a2b, 2 is b2a
                    direction: u8::from_le_bytes(*direction_buf),
                }
            }
            _ => return Err(AmmError::InvalidInstruction.into()),
        })
    }

    fn unpack_pubkey(input: &[u8]) -> Result<Pubkey, ProgramError> {
        if input.len() >= 32 {
            let (key, _rest) = input.split_at(32);
            let pk = Pubkey::new(key);
            Ok(pk)
        } else {
            Err(AmmError::InvalidInstruction.into())
        }
    }

    // pack function to pack a SapInstruction enum into a byte array for test convenience
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            // pack initialize
            &Self::Initialize {
                nonce,
                fee_1,
                fee_2,
                fee_3,
                fee_4,
                fee_5,
                amount_a,
                amount_b,
            } => {
                buf.push(0);
                buf.extend_from_slice(&nonce.to_le_bytes());
                buf.extend_from_slice(&fee_1.to_le_bytes());
                buf.extend_from_slice(&fee_2.to_le_bytes());
                buf.extend_from_slice(&fee_3.to_le_bytes());
                buf.extend_from_slice(&fee_5.to_le_bytes());
                buf.extend_from_slice(&amount_a.to_le_bytes());
                buf.extend_from_slice(&amount_b.to_le_bytes());
            }
            // pack update pool
            &Self::UpdatePool {} => {
                buf.push(1);
            }
            // pack update status
            &Self::UpdateStatus { status } => {
                buf.push(2);
                buf.extend_from_slice(&status.to_le_bytes());
            }

            // pack swap
            &Self::Swap { amount, direction } => {
                buf.push(10);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&direction.to_le_bytes());
            }
        }
        buf
    }
}
