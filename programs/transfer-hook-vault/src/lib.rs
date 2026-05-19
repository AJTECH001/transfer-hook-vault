#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod vault_hooks;

pub use errors::*;
pub use events::*;
pub use state::*;
pub use instructions::*;
pub use vault_hooks::*;

declare_id!("9eAQgQ8P3HFkbFW178Rf1bRg5L2vGyrNqGJ8kwwmM8Xf");

#[program]
pub mod transfer_hook_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>, initial_supply: u64) -> Result<()> {
        ctx.accounts.initialize_vault(ctx.bumps, initial_supply)
    }

    pub fn add_to_whitelist_vec(ctx: Context<AddToWhitelistVec>, user: Pubkey, amount: u64) -> Result<()> {
        instructions::add_to_whitelist::add_to_whitelist_vec(ctx, user, amount)
    }

    pub fn add_to_whitelist_pda(ctx: Context<AddToWhitelistPDA>, user: Pubkey, amount: u64) -> Result<()> {
        instructions::add_to_whitelist::add_to_whitelist_pda(ctx, user, amount)
    }

    pub fn remove_from_whitelist_vec(ctx: Context<RemoveFromWhitelistVec>, user: Pubkey) -> Result<()> {
        instructions::remove_from_whitelist::remove_from_whitelist_vec(ctx, user)
    }

    pub fn remove_from_whitelist_pda(ctx: Context<RemoveFromWhitelistPDA>, user: Pubkey) -> Result<()> {
        instructions::remove_from_whitelist::remove_from_whitelist_pda(ctx, user)
    }

    pub fn initialize_extra_account_meta_list(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {
        vault_hooks::initialize_extra_account_meta_list(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn transfer_hook(ctx: Context<TransferHookAccounts>, amount: u64) -> Result<()> {
        vault_hooks::execute_transfer_hook(ctx.program_id, ctx.remaining_accounts, &amount.to_le_bytes())
    }

    pub fn fallback<'info>(
        _program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        if data.len() >= 8 && data[..8] == [105, 37, 101, 197, 75, 251, 102, 26] {
            vault_hooks::execute_transfer_hook(_program_id, accounts, data)
        } else {
            Err(ProgramError::InvalidInstructionData.into())
        }
    }
}
