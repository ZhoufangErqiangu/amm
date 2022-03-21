//! State transition types

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

/// Initially launching with USDT/BTC, USDT/ETH, USDT/SOL, SYP
pub const TOKEN_NUM: usize = 6; // 6 tokens
pub const LIST_NUM: usize = 3;
pub const INIT_NUM: usize = 3;
/// Ido pool status.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PoolStatus {
    /// If the account has not been initialized, the enum will be 0
    Uninitialized = 0,
    PreMint = 1,
    Lock = 2,
    FreeMintBurn = 3,
}

/// Initialized program details.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SapPool {
    // pool status
    // 0 is not init
    // 1 is pre mint
    // 2 is lock
    // 3 is free mint burn
    pub status: u64,
    // Nonce used in program address.
    pub nonce: u64,
    // every mint need pay some syp
    // no use
    pub syp_percent: u64,
    // owner address
    pub owner_pubkey: Pubkey,
    // sap pool mint address
    pub sap_mint_l1_pubkey: Pubkey,
    pub sap_mint_l2_pubkey: Pubkey,
    // mint to trace
    pub sap_mint_l3_pubkey: Pubkey,
    // pre mint
    pub pre_mint_l1_pubkey: Pubkey,
    pub pre_mint_l2_pubkey: Pubkey,
    // pool manager
    pub manager_pubkey: Pubkey,
    // receive every mint syp
    // no use
    pub syp_vault: Pubkey,
    // oracle price pubkey
    pub oracle_price_pubkey_list: [Pubkey; TOKEN_NUM],
    // sap token pool list
    pub token_list: [Pubkey; TOKEN_NUM],
    // sap token assets related vault list
    pub token_assets_vault_pubkeys: [Pubkey; TOKEN_NUM],
    // sap init price
    pub sap_init_price: f64,
    // sap init ts
    pub sap_init_ts: i64,
    // redeem time lock
    pub time_lock: i64,
    // sap_pre_mint_l1_amount
    pub sap_pre_mint_l1_amount: u64,
    // sap_pre_mint_l2_amount
    pub sap_pre_mint_l2_amount: u64,
    // sap pre mint claim rate l1
    pub sap_pre_mint_l1_claim_rate: f64,
    // sap pre mint claim rate l2
    pub sap_pre_mint_l2_claim_rate: f64,
    // save fee data
    pub fee: f64,
    pub performance_fee: f64,
}

impl Sealed for SapPool {}
impl IsInitialized for SapPool {
    fn is_initialized(&self) -> bool {
        true
        // self.owner_pubkey != NULL_PUBKEY
    }
}
impl Pack for SapPool {
    const LEN: usize = 8 * 3 + 32 * 8 + 32 * 3 * TOKEN_NUM + 8 * 9;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        const LEN: usize = 8 * 3 + 32 * 8 + 32 * 3 * TOKEN_NUM + 8 * 9;
        let src = array_ref![src, 0, LEN];
        let (
            status_buf,
            nonce_buf,
            syp_percent_buf,
            owner_pubkey_buf,
            sap_mint_l1_pubkey_buf,
            sap_mint_l2_pubkey_buf,
            sap_mint_l3_pubkey_buf,
            pre_mint_l1_pubkey_buf,
            pre_mint_l2_pubkey_buf,
            manager_pubkey_buf,
            syp_vault_buf,
            oracle_price_pubkey_list_buf,
            token_list_buf,
            token_assets_vault_pubkeys_buf,
            sap_init_price_buf,
            sap_init_ts_buf,
            time_lock_buf,
            sap_pre_mint_l1_amount_buf,
            sap_pre_mint_l2_amount_buf,
            sap_pre_mint_l1_claim_rate_buf,
            sap_pre_mint_l2_claim_rate_buf,
            fee_buf,
            performance_fee_buf,
        ) = array_refs![
            src,
            8,
            8,
            8,
            32,
            32,
            32,
            32,
            32,
            32,
            32,
            32,
            32 * TOKEN_NUM,
            32 * TOKEN_NUM,
            32 * TOKEN_NUM,
            8,
            8,
            8,
            8,
            8,
            8,
            8,
            8,
            8
        ];

        let mut token_list_upacked: [Pubkey; TOKEN_NUM] = Default::default();
        let mut token_assets_vault_pubkeys_unpacked: [Pubkey; TOKEN_NUM] = Default::default();
        let mut oracle_price_pubkey_list_unpacked: [Pubkey; TOKEN_NUM] = Default::default();

        for (src, dst) in oracle_price_pubkey_list_buf
            .chunks(32)
            .zip(oracle_price_pubkey_list_unpacked.iter_mut())
        {
            *dst = Pubkey::new(src);
        }

        for (src, dst) in token_list_buf.chunks(32).zip(token_list_upacked.iter_mut()) {
            *dst = Pubkey::new(src);
        }
        for (src, dst) in token_assets_vault_pubkeys_buf
            .chunks(32)
            .zip(token_assets_vault_pubkeys_unpacked.iter_mut())
        {
            *dst = Pubkey::new(src);
        }

        Ok(SapPool {
            status: u64::from_le_bytes(*status_buf),
            nonce: u64::from_le_bytes(*nonce_buf),
            syp_percent: u64::from_le_bytes(*syp_percent_buf),
            owner_pubkey: Pubkey::new_from_array(*owner_pubkey_buf),
            sap_mint_l1_pubkey: Pubkey::new_from_array(*sap_mint_l1_pubkey_buf),
            sap_mint_l2_pubkey: Pubkey::new_from_array(*sap_mint_l2_pubkey_buf),
            sap_mint_l3_pubkey: Pubkey::new_from_array(*sap_mint_l3_pubkey_buf),
            pre_mint_l1_pubkey: Pubkey::new_from_array(*pre_mint_l1_pubkey_buf),
            pre_mint_l2_pubkey: Pubkey::new_from_array(*pre_mint_l2_pubkey_buf),
            manager_pubkey: Pubkey::new_from_array(*manager_pubkey_buf),
            syp_vault: Pubkey::new_from_array(*syp_vault_buf),
            oracle_price_pubkey_list: oracle_price_pubkey_list_unpacked,
            token_list: token_list_upacked,
            token_assets_vault_pubkeys: token_assets_vault_pubkeys_unpacked,
            sap_init_price: f64::from_le_bytes(*sap_init_price_buf),
            sap_init_ts: i64::from_le_bytes(*sap_init_ts_buf),
            time_lock: i64::from_le_bytes(*time_lock_buf),
            sap_pre_mint_l1_amount: u64::from_le_bytes(*sap_pre_mint_l1_amount_buf),
            sap_pre_mint_l2_amount: u64::from_le_bytes(*sap_pre_mint_l2_amount_buf),
            sap_pre_mint_l1_claim_rate: f64::from_le_bytes(*sap_pre_mint_l1_claim_rate_buf),
            sap_pre_mint_l2_claim_rate: f64::from_le_bytes(*sap_pre_mint_l2_claim_rate_buf),
            fee: f64::from_le_bytes(*fee_buf),
            performance_fee: f64::from_le_bytes(*performance_fee_buf),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        const LEN: usize = 8 * 3 + 32 * 8 + 32 * 3 * TOKEN_NUM + 8 * 9;
        let dst = array_mut_ref![dst, 0, LEN];
        let (
            status_buf,
            nonce_buf,
            syp_percent_buf,
            owner_pubkey_buf,
            sap_mint_l1_pubkey_buf,
            sap_mint_l2_pubkey_buf,
            sap_mint_l3_pubkey_buf,
            pre_mint_l1_pubkey_buf,
            pre_mint_l2_pubkey_buf,
            manager_pubkey_buf,
            syp_vault_buf,
            oracle_price_pubkey_list_buf,
            token_list_buf,
            token_assets_vault_pubkeys_buf,
            sap_init_price_buf,
            sap_init_ts_buf,
            time_lock_buf,
            sap_pre_mint_l1_amount_buf,
            sap_pre_mint_l2_amount_buf,
            sap_pre_mint_l1_claim_rate_buf,
            sap_pre_mint_l2_claim_rate_buf,
            fee_buf,
            performance_fee_buf,
        ) = mut_array_refs![
            dst,
            8,
            8,
            8,
            32,
            32,
            32,
            32,
            32,
            32,
            32,
            32,
            32 * TOKEN_NUM,
            32 * TOKEN_NUM,
            32 * TOKEN_NUM,
            8,
            8,
            8,
            8,
            8,
            8,
            8,
            8,
            8
        ];
        *status_buf = self.status.to_le_bytes();
        *nonce_buf = self.nonce.to_le_bytes();
        *syp_percent_buf = self.syp_percent.to_le_bytes();
        owner_pubkey_buf.copy_from_slice(self.owner_pubkey.as_ref());
        sap_mint_l1_pubkey_buf.copy_from_slice(self.sap_mint_l1_pubkey.as_ref());
        sap_mint_l2_pubkey_buf.copy_from_slice(self.sap_mint_l2_pubkey.as_ref());
        sap_mint_l3_pubkey_buf.copy_from_slice(self.sap_mint_l3_pubkey.as_ref());
        pre_mint_l1_pubkey_buf.copy_from_slice(self.pre_mint_l1_pubkey.as_ref());
        pre_mint_l2_pubkey_buf.copy_from_slice(self.pre_mint_l2_pubkey.as_ref());
        manager_pubkey_buf.copy_from_slice(self.manager_pubkey.as_ref());
        syp_vault_buf.copy_from_slice(self.syp_vault.as_ref());
        *sap_init_price_buf = self.sap_init_price.to_le_bytes();
        *sap_init_ts_buf = self.sap_init_ts.to_le_bytes();
        *time_lock_buf = self.time_lock.to_le_bytes();
        *sap_pre_mint_l1_amount_buf = self.sap_pre_mint_l1_amount.to_le_bytes();
        *sap_pre_mint_l2_amount_buf = self.sap_pre_mint_l2_amount.to_le_bytes();
        *sap_pre_mint_l1_claim_rate_buf = self.sap_pre_mint_l1_claim_rate.to_le_bytes();
        *sap_pre_mint_l2_claim_rate_buf = self.sap_pre_mint_l2_claim_rate.to_le_bytes();
        *fee_buf = self.fee.to_le_bytes();
        *performance_fee_buf = self.performance_fee.to_le_bytes();

        for (i, src) in self.token_list.iter().enumerate() {
            let dst_array = array_mut_ref![token_list_buf, 32 * i, 32];
            dst_array.copy_from_slice(src.as_ref());
        }

        for (i, src) in self.token_assets_vault_pubkeys.iter().enumerate() {
            let dst_array = array_mut_ref![token_assets_vault_pubkeys_buf, 32 * i, 32];
            dst_array.copy_from_slice(src.as_ref());
        }
        for (i, src) in self.oracle_price_pubkey_list.iter().enumerate() {
            let dst_array = array_mut_ref![oracle_price_pubkey_list_buf, 32 * i, 32];
            dst_array.copy_from_slice(src.as_ref());
        }
    }
}

/// user sap data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UserSapMember {
    pub sap_pool: Pubkey,
    pub user_wallet: Pubkey,
    pub user_sap_account: Pubkey,
    pub user_reward_spt: Pubkey,
    pub nonce: u64,
    pub ltype: u64,
    pub sap_cost: u64,
    pub last_mint_ts: i64,
}
impl Sealed for UserSapMember {}
impl IsInitialized for UserSapMember {
    fn is_initialized(&self) -> bool {
        true
    }
}
impl Pack for UserSapMember {
    const LEN: usize = 32 * 4 + 8 * 4;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        const LEN: usize = 32 * 4 + 8 * 4;
        let src = array_ref![src, 0, LEN];
        let (
            sap_pool_buf,
            user_wallet_buf,
            user_sap_account_buf,
            user_reward_spt_buf,
            nonce_buf,
            ltype_buf,
            sap_cost_buf,
            last_mint_ts_buf,
        ) = array_refs![src, 32, 32, 32, 32, 8, 8, 8, 8];
        Ok(UserSapMember {
            sap_pool: Pubkey::new_from_array(*sap_pool_buf),
            user_wallet: Pubkey::new_from_array(*user_wallet_buf),
            user_sap_account: Pubkey::new_from_array(*user_sap_account_buf),
            user_reward_spt: Pubkey::new_from_array(*user_reward_spt_buf),
            nonce: u64::from_le_bytes(*nonce_buf),
            ltype: u64::from_le_bytes(*ltype_buf),
            sap_cost: u64::from_le_bytes(*sap_cost_buf),
            last_mint_ts: i64::from_le_bytes(*last_mint_ts_buf),
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        const LEN: usize = 32 * 4 + 8 * 4;
        let dst = array_mut_ref![dst, 0, LEN];
        let (
            sap_pool_buf,
            user_wallet_buf,
            user_sap_account_buf,
            user_reward_spt_buf,
            nonce_buf,
            ltype_buf,
            sap_cost_buf,
            last_mint_ts_buf,
        ) = mut_array_refs![dst, 32, 32, 32, 32, 8, 8, 8, 8];
        sap_pool_buf.copy_from_slice(self.sap_pool.as_ref());
        user_wallet_buf.copy_from_slice(self.user_wallet.as_ref());
        user_sap_account_buf.copy_from_slice(self.user_sap_account.as_ref());
        user_reward_spt_buf.copy_from_slice(self.user_reward_spt.as_ref());
        *nonce_buf = self.nonce.to_le_bytes();
        *ltype_buf = self.ltype.to_le_bytes();
        *sap_cost_buf = self.sap_cost.to_le_bytes();
        *last_mint_ts_buf = self.last_mint_ts.to_le_bytes();
    }
}

// serem dex state here

// oracle pyth state here

// public key of 11111111111111111111111111111111
pub const NULL_PUBKEY: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);
