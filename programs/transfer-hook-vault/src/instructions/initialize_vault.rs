use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::state::{VaultState, DECIMALS};

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [b"vault-state"],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = admin,
        mint::decimals = DECIMALS,
        mint::authority = vault_state,
        mint::token_program = token_program,
        extensions::transfer_hook::program_id = crate::ID,
        extensions::transfer_hook::authority = vault_state,
        extensions::metadata_pointer::authority = vault_state,
        extensions::metadata_pointer::metadata_address = mint.key(),
        seeds = [b"vault-mint"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = vault_state,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeVault<'info> {
    pub fn initialize_vault(
        &mut self,
        bumps: InitializeVaultBumps,
        initial_supply: u64,
    ) -> Result<()> {
        // Store config in the VaultState PDA.
        self.vault_state.set_inner(VaultState {
            admin: self.admin.key(),
            mint: self.mint.key(),
            vault_token_account: self.vault_token_account.key(),
            bump: bumps.vault_state,
            mint_bump: bumps.mint,
            whitelist_vec: Vec::new(),
        });

        let seeds = &[b"vault-state".as_ref(), &[self.vault_state.bump]];
        let signer_seeds = &[&seeds[..]];

        // Mint the initial supply into the vault itself.
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.key(),
                MintTo {
                    mint: self.mint.to_account_info(),
                    to: self.vault_token_account.to_account_info(),
                    authority: self.vault_state.to_account_info(),
                },
                signer_seeds,
            ),
            initial_supply,
        )?;

        msg!(
            "Vault initialized. Mint: {}. Supply: {}.",
            self.mint.key(),
            initial_supply
        );

        Ok(())
    }
}
