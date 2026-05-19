use anchor_lang::prelude::*;

use crate::{
    errors::VaultError,
    events::WhitelistUpdateEvent,
    state::{VaultState, WhitelistAccount, WhitelistEntry},
};

#[derive(Accounts)]
pub struct AddToWhitelistVec<'info> {
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
pub struct AddToWhitelistPDA<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"vault-state"],
        bump = vault_state.bump,
        has_one = admin,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = admin,
        space = 8 + WhitelistAccount::INIT_SPACE,
        seeds = [WhitelistAccount::SEED_PREFIX, user.as_ref()],
        bump
    )]
    pub whitelist_pda: Account<'info, WhitelistAccount>,

    pub system_program: Program<'info, System>,
}

pub fn add_to_whitelist_vec(ctx: Context<AddToWhitelistVec>, user: Pubkey, amount: u64) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    
    require!(
        vault_state.whitelist_vec.len() < 100,
        VaultError::WhitelistFull
    );

    require!(
        !vault_state.whitelist_vec.iter().any(|e| e.user == user),
        VaultError::AlreadyWhitelisted
    );

    vault_state.whitelist_vec.push(WhitelistEntry { user, amount });

    emit!(WhitelistUpdateEvent { user, is_added: true });

    msg!("Added {} to Vec whitelist. Total: {}", user, vault_state.whitelist_vec.len());
    Ok(())
}

pub fn add_to_whitelist_pda(ctx: Context<AddToWhitelistPDA>, user: Pubkey, amount: u64) -> Result<()> {
    let whitelist_pda = &mut ctx.accounts.whitelist_pda;
    whitelist_pda.user = user;
    whitelist_pda.amount = amount;
    whitelist_pda.bump = ctx.bumps.whitelist_pda;

    emit!(WhitelistUpdateEvent { user, is_added: true });

    msg!("Added {} to PDA whitelist.", user);
    Ok(())
}
