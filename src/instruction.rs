//! Instruction types
#![allow(clippy::too_many_arguments)]

use crate::error::AmmError;
// use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use arrayref::{array_ref, array_refs};
use solana_program::program_error::ProgramError;
use std::{fmt, mem::size_of};

/// swap direction
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    A2B,
    B2A,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status: String = match self {
            Direction::A2B => "A to B".to_string(),
            Direction::B2A => "B to A".to_string(),
        };
        write!(f, "{}", status)
    }
}

/// Instructions supported by the token program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum AmmInstruction {
    Initialize {
        nonce: u8,
        fee: u64,
        amount_a: u64,
        amount_b: u64,
        tolerance: u64,
    },
    UpdateStatus {
        status: u8,
    },
    UpdateTolerance {
        tolerance: u64,
    },
    Swap {
        amount: u64,
        direction: Direction,
    },
    WithdrawalFee {},
    Terminate {},
}

impl AmmInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(crate::error::AmmError::InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let data = array_ref![rest, 0, 1 + 8 * 4];
                let (nonce_buf, fee_buf, amount_a_buf, amount_b_buf, tolerance_buf) =
                    array_refs![data, 1, 8, 8, 8, 8];
                Self::Initialize {
                    nonce: u8::from_le_bytes(*nonce_buf),
                    fee: u64::from_le_bytes(*fee_buf),
                    amount_a: u64::from_le_bytes(*amount_a_buf),
                    amount_b: u64::from_le_bytes(*amount_b_buf),
                    tolerance: u64::from_le_bytes(*tolerance_buf),
                }
            }
            2 => {
                let data = array_ref![rest, 0, 1];
                Self::UpdateStatus {
                    status: u8::from_le_bytes(*data),
                }
            }
            3 => {
                let data = array_ref![rest, 0, 8];
                Self::UpdateTolerance {
                    tolerance: u64::from_le_bytes(*data),
                }
            }
            9 => Self::Terminate {},

            10 => {
                let data = array_ref![rest, 0, 8 + 1];
                let (amount_buf, direction_buf) = array_refs![data, 8, 1];
                // 1 is a2b, 2 is b2a
                let direction = match u8::from_le_bytes(*direction_buf) {
                    1 => Direction::A2B,
                    2 => Direction::B2A,
                    _ => return Err(AmmError::InvalidDirection.into()),
                };

                Self::Swap {
                    amount: u64::from_le_bytes(*amount_buf),
                    direction: direction,
                }
            }

            80 => Self::WithdrawalFee {},

            _ => return Err(AmmError::InvalidInstruction.into()),
        })
    }

    // pack function to pack a AmmInstruction enum into a byte array for test convenience
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::Initialize {
                nonce,
                fee,
                amount_a,
                amount_b,
                tolerance,
            } => {
                buf.push(0);
                buf.extend_from_slice(&nonce.to_le_bytes());
                buf.extend_from_slice(&fee.to_le_bytes());
                buf.extend_from_slice(&amount_a.to_le_bytes());
                buf.extend_from_slice(&amount_b.to_le_bytes());
                buf.extend_from_slice(&tolerance.to_le_bytes());
            }
            &Self::UpdateStatus { status } => {
                buf.push(2);
                buf.extend_from_slice(&status.to_le_bytes());
            }
            &Self::UpdateTolerance { tolerance } => {
                buf.push(3);
                buf.extend_from_slice(&tolerance.to_le_bytes());
            }
            &Self::Terminate {} => {
                buf.push(9);
            }

            &Self::Swap { amount, direction } => {
                buf.push(10);
                buf.extend_from_slice(&amount.to_le_bytes());
                // 1 is a2b, 2 is b2a
                match direction {
                    Direction::A2B => buf.push(1),
                    Direction::B2A => buf.push(2),
                }
            }

            &Self::WithdrawalFee {} => {
                buf.push(80);
            }
        }
        buf
    }
}
