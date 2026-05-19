use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub bump: u8,
    pub mint_bump: u8,
    
    // Approach 1: Vec-based whitelist
    // We keep this for comparison purposes as requested.
    #[max_len(100)]
    pub whitelist_vec: Vec<WhitelistEntry>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct WhitelistEntry {
    pub user: Pubkey,
    pub amount: u64, // Amount they are allowed to deposit/withdraw or current balance
}
