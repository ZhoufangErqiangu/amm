// #![deny(missing_docs)]

//! A program for creating pools of Solana stakes managed by a Stake-o-Matic

mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

// Export current SDK types for downstream users building with a different SDK version
pub use solana_program;
