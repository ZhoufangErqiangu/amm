//! State transition types
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

/// amm pool
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AmmPool {
    // pool status
    // 0 is not init
    // 1 is nomal
    // 2 is lock
    pub status: u8,
    // Nonce used in program address.
    pub nonce: u8,
    pub ka: u64,
    pub kb: u64,
    // fee rate
    pub fee_1: f64,
    pub fee_2: f64,
    pub fee_3: f64,
    pub fee_4: f64,
    pub fee_5: f64,
    // owner address
    pub owner: Pubkey,
    // swap mint public key
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    // vaults for swap mint
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    // fee receiver
    pub fee_vault: Pubkey,
    pub fee_receiver_1: Pubkey,
    pub fee_receiver_2: Pubkey,
    pub fee_receiver_3: Pubkey,
    pub fee_receiver_4: Pubkey,
    pub fee_receiver_5: Pubkey,
    pub fee_mint: Pubkey,
}

impl Sealed for AmmPool {}
impl IsInitialized for AmmPool {
    fn is_initialized(&self) -> bool {
        true
        // self.owner_pubkey != NULL_PUBKEY
    }
}
impl Pack for AmmPool {
    const LEN: usize = 1 * 2 + 8 * 7 + 32 * 12;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        const LEN: usize = 1 * 2 + 8 * 7 + 32 * 12;
        let src = array_ref![src, 0, LEN];
        let (
            status_buf,
            nonce_buf,
            ka_buf,
            kb_buf,
            fee_1_buf,
            fee_2_buf,
            fee_3_buf,
            fee_4_buf,
            fee_5_buf,
            owner_buf,
            mint_a_buf,
            mint_b_buf,
            vault_a_buf,
            vault_b_buf,
            fee_vault_buf,
            fee_receiver_1_buf,
            fee_receiver_2_buf,
            fee_receiver_3_buf,
            fee_receiver_4_buf,
            fee_receiver_5_buf,
            fee_mint_buf,
        ) = array_refs![
            src, 1, 1, 8, 8, 8, 8, 8, 8, 8, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32
        ];

        Ok(AmmPool {
            status: u8::from_le_bytes(*status_buf),
            nonce: u8::from_le_bytes(*nonce_buf),
            ka: u64::from_le_bytes(*ka_buf),
            kb: u64::from_le_bytes(*kb_buf),
            fee_1: f64::from_le_bytes(*fee_1_buf),
            fee_2: f64::from_le_bytes(*fee_2_buf),
            fee_3: f64::from_le_bytes(*fee_3_buf),
            fee_4: f64::from_le_bytes(*fee_4_buf),
            fee_5: f64::from_le_bytes(*fee_5_buf),
            owner: Pubkey::new_from_array(*owner_buf),
            mint_a: Pubkey::new_from_array(*mint_a_buf),
            mint_b: Pubkey::new_from_array(*mint_b_buf),
            vault_a: Pubkey::new_from_array(*vault_a_buf),
            vault_b: Pubkey::new_from_array(*vault_b_buf),
            fee_vault: Pubkey::new_from_array(*fee_receiver_1_buf),
            fee_receiver_1: Pubkey::new_from_array(*fee_vault_buf),
            fee_receiver_2: Pubkey::new_from_array(*fee_receiver_2_buf),
            fee_receiver_3: Pubkey::new_from_array(*fee_receiver_3_buf),
            fee_receiver_4: Pubkey::new_from_array(*fee_receiver_4_buf),
            fee_receiver_5: Pubkey::new_from_array(*fee_receiver_5_buf),
            fee_mint: Pubkey::new_from_array(*fee_mint_buf),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        const LEN: usize = 1 * 2 + 8 * 7 + 32 * 11;
        let dst = array_mut_ref![dst, 0, LEN];
        let (
            status_buf,
            nonce_buf,
            ka_buf,
            kb_buf,
            fee_1_buf,
            fee_2_buf,
            fee_3_buf,
            fee_4_buf,
            fee_5_buf,
            owner_buf,
            mint_a_buf,
            mint_b_buf,
            vault_a_buf,
            vault_b_buf,
            fee_vault_buf,
            fee_receiver_1_buf,
            fee_receiver_2_buf,
            fee_receiver_3_buf,
            fee_receiver_4_buf,
            fee_receiver_5_buf,
            fee_mint_buf,
        ) = mut_array_refs![
            dst, 1, 1, 8, 8, 8, 8, 8, 8, 8, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32
        ];
        *status_buf = self.status.to_le_bytes();
        *nonce_buf = self.nonce.to_le_bytes();
        *ka_buf = self.ka.to_le_bytes();
        *kb_buf = self.kb.to_le_bytes();
        *fee_1_buf = self.fee_1.to_le_bytes();
        *fee_2_buf = self.fee_2.to_le_bytes();
        *fee_3_buf = self.fee_3.to_le_bytes();
        *fee_4_buf = self.fee_4.to_le_bytes();
        *fee_5_buf = self.fee_5.to_le_bytes();
        owner_buf.copy_from_slice(self.owner.as_ref());
        mint_a_buf.copy_from_slice(self.mint_a.as_ref());
        mint_b_buf.copy_from_slice(self.mint_b.as_ref());
        vault_a_buf.copy_from_slice(self.vault_a.as_ref());
        vault_b_buf.copy_from_slice(self.vault_b.as_ref());
        fee_vault_buf.copy_from_slice(self.fee_vault.as_ref());
        fee_receiver_1_buf.copy_from_slice(self.fee_receiver_1.as_ref());
        fee_receiver_2_buf.copy_from_slice(self.fee_receiver_2.as_ref());
        fee_receiver_3_buf.copy_from_slice(self.fee_receiver_3.as_ref());
        fee_receiver_4_buf.copy_from_slice(self.fee_receiver_4.as_ref());
        fee_receiver_5_buf.copy_from_slice(self.fee_receiver_5.as_ref());
        fee_mint_buf.copy_from_slice(self.fee_mint.as_ref());
    }
}
