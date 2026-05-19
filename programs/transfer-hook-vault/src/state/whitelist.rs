use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct WhitelistAccount {
    pub user: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

impl WhitelistAccount {
    pub const SEED_PREFIX: &'static [u8] = b"whitelist";
}
