use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Caller is not on the whitelist")]
    NotWhitelisted,
    #[msg("Whitelist is full — max 100 addresses (Approach 1 limit)")]
    WhitelistFull,
    #[msg("Address is already on the whitelist")]
    AlreadyWhitelisted,
    #[msg("Address is not on the whitelist")]
    NotInWhitelist,
    #[msg("Insufficient tokens in the vault")]
    InsufficientFunds,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Invalid PDA derivation")]
    InvalidPDA,
    #[msg("Overflow occurred")]
    Overflow,
}
