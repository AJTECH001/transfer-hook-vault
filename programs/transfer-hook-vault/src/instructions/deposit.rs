use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{
    errors::VaultError,
    events::DepositEvent,
    state::{VaultState, WhitelistAccount, DECIMALS},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault-state"],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        address = vault_state.mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
        token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault_state,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: ExtraAccountMetaList PDA for transfer hook
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    /// CHECK: Whitelist PDA (optional)
    #[account(
        seeds = [WhitelistAccount::SEED_PREFIX, user.key().as_ref()],
        bump,
    )]
    pub whitelist_pda: UncheckedAccount<'info>,

    /// CHECK: The Transfer Hook program (this program)
    #[account(address = crate::ID)]
    pub transfer_hook_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        // 1. Check Approach 1: Vec in VaultState
        let is_in_vec = self.vault_state.whitelist_vec.iter().any(|e| e.user == self.user.key());

        // 2. Check Approach 2: PDA-per-user
        let is_in_pda = !self.whitelist_pda.data_is_empty();

        require!(is_in_vec || is_in_pda, VaultError::NotWhitelisted);

        // Manually build the TransferChecked instruction to ensure extra accounts are included.
        let mut accounts = vec![
            AccountMeta::new(self.user_token_account.key(), false),
            AccountMeta::new_readonly(self.mint.key(), false),
            AccountMeta::new(self.vault_token_account.key(), false),
            AccountMeta::new_readonly(self.user.key(), true),
        ];

        // Add remaining accounts for the transfer hook
        accounts.push(AccountMeta::new_readonly(self.extra_account_meta_list.key(), false));
        accounts.push(AccountMeta::new_readonly(self.vault_state.key(), false));
        accounts.push(AccountMeta::new_readonly(self.whitelist_pda.key(), false));

        let ix = anchor_spl::token_2022::spl_token_2022::instruction::transfer_checked(
            &self.token_program.key(),
            &self.user_token_account.key(),
            &self.mint.key(),
            &self.vault_token_account.key(),
            &self.user.key(),
            &[],
            amount,
            DECIMALS,
        )?;

        // We need to replace the accounts in the instruction with our augmented list.
        let mut augmented_ix = ix;
        augmented_ix.accounts = accounts;

        anchor_lang::solana_program::program::invoke(
            &augmented_ix,
            &[
                self.user_token_account.to_account_info(),
                self.mint.to_account_info(),
                self.vault_token_account.to_account_info(),
                self.user.to_account_info(),
                self.extra_account_meta_list.to_account_info(),
                self.vault_state.to_account_info(),
                self.whitelist_pda.to_account_info(),
                self.transfer_hook_program.to_account_info(), // Still need the program info for the runtime
                self.token_program.to_account_info(),
            ],
        )?;

        emit!(DepositEvent {
            user: self.user.key(),
            amount,
        });

        msg!("Deposited {} tokens into vault", amount);
        Ok(())
    }
}
