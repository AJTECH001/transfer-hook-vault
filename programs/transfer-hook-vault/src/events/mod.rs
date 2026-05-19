use anchor_lang::prelude::*;

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WhitelistUpdateEvent {
    pub user: Pubkey,
    pub is_added: bool,
}
