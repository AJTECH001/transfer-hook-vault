use anchor_lang::prelude::*;

use crate::{
    errors::VaultError,
    events::WhitelistUpdateEvent,
    state::{VaultState, WhitelistAccount},
};

#[derive(Accounts)]
pub struct RemoveFromWhitelistVec<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault-state"],
        bump = vault_state.bump,
        has_one = admin,
    )]
    pub vault_state: Account<'info, VaultState>,
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct RemoveFromWhitelistPDA<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"vault-state"],
        bump = vault_state.bump,
        has_one = admin,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        close = admin,
        seeds = [WhitelistAccount::SEED_PREFIX, user.as_ref()],
        bump = whitelist_pda.bump,
    )]
    pub whitelist_pda: Account<'info, WhitelistAccount>,
}

pub fn remove_from_whitelist_vec(ctx: Context<RemoveFromWhitelistVec>, user: Pubkey) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    
    let index = vault_state.whitelist_vec.iter().position(|e| e.user == user)
        .ok_or(VaultError::NotInWhitelist)?;

    vault_state.whitelist_vec.remove(index);

    emit!(WhitelistUpdateEvent { user, is_added: false });

    msg!("Removed {} from Vec whitelist. Total: {}", user, vault_state.whitelist_vec.len());
    Ok(())
}

pub fn remove_from_whitelist_pda(_ctx: Context<RemoveFromWhitelistPDA>, user: Pubkey) -> Result<()> {
    // Account is closed via the `close` constraint.
    emit!(WhitelistUpdateEvent { user, is_added: false });

    msg!("Removed {} from PDA whitelist (account closed).", user);
    Ok(())
}
