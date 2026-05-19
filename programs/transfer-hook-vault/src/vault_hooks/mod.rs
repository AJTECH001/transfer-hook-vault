use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_error::ProgramError;
use anchor_spl::token_interface::{Mint, TokenAccount};
use anchor_spl::token_2022::spl_token_2022;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use solana_program::program_pack::Pack;

use crate::{
    errors::VaultError,
    state::{VaultState, WhitelistAccount},
};

pub fn execute_transfer_hook(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<()> {
    msg!("Transfer hook executing via fallback...");
    
    // The amount is in the data after the 8-byte discriminator.
    if data.len() < 16 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&data[8..16]);
    let amount = u64::from_le_bytes(amount_bytes);

    // Standard accounts for Execute:
    // 0: source
    // 1: mint
    // 2: destination
    // 3: owner
    // 4: extra_account_meta_list
    // 5+: extra accounts
    
    let source_token_info = &accounts[0];
    let destination_token_info = &accounts[2];
    let vault_state_info = &accounts[5];
    let whitelist_pda_info = &accounts[6];

    execute_logic(
        program_id,
        source_token_info,
        destination_token_info,
        vault_state_info,
        whitelist_pda_info,
        amount,
    )
}

fn execute_logic(
    _program_id: &Pubkey,
    source_token_info: &AccountInfo,
    destination_token_info: &AccountInfo,
    vault_state_info: &AccountInfo,
    whitelist_pda_info: &AccountInfo,
    amount: u64,
) -> Result<()> {
    // Unpack Token-2022 accounts safely without discriminator
    let source_token = spl_token_2022::state::Account::unpack(&source_token_info.data.borrow())?;
    let destination_token = spl_token_2022::state::Account::unpack(&destination_token_info.data.borrow())?;
    
    let vault_state = VaultState::try_deserialize(&mut &vault_state_info.data.borrow()[..])?;

    let is_deposit = destination_token_info.key() == vault_state.vault_token_account;
    let is_withdrawal = source_token_info.key() == vault_state.vault_token_account;

    msg!("Transfer Hook: amount={}, deposit={}, withdraw={}", amount, is_deposit, is_withdrawal);

    if !is_deposit && !is_withdrawal {
        msg!("Not a vault interaction, skipping whitelist check.");
        return Ok(());
    }

    let user_to_check = if is_deposit {
        source_token.owner
    } else {
        destination_token.owner
    };

    // 1. Check Approach 1: Vec in VaultState
    let is_in_vec = vault_state.whitelist_vec.iter().any(|e| e.user == user_to_check);

    // 2. Check Approach 2: PDA-per-user
    let is_in_pda = !whitelist_pda_info.data_is_empty();

    if !is_in_vec && !is_in_pda {
        msg!("User {} is not whitelisted!", user_to_check);
        return Err(VaultError::NotWhitelisted.into());
    }

    msg!("Whitelist check passed for user {}.", user_to_check);
    Ok(())
}

#[derive(Accounts)]
pub struct TransferHookAccounts<'info> {
    #[account(
        token::mint = mint,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        token::mint = mint,
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: source token account owner
    pub owner: UncheckedAccount<'info>,
    /// CHECK: ExtraAccountMetaList PDA
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    
    pub vault_state: Account<'info, VaultState>,
    
    /// CHECK: Whitelist PDA (optional, might be empty if not created yet)
    pub whitelist_pda: UncheckedAccount<'info>,
}

// ─── TEACHING MOMENT: Extra Account Metas ────────────────────────────────────
//
// Transfer Hooks are "blind" by default — they only get source, destination,
// mint, and owner. If you need EXTRA accounts (like our whitelist PDA),
// you must define them in an ExtraAccountMetaList account.
//
// The Token-2022 program reads this list and automatically adds these accounts
// to the CPI call.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList PDA
    #[account(
        init,
        payer = payer,
        space = ExtraAccountMetaList::size_of(2)?, // We need 2 extra accounts
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_extra_account_meta_list(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {
    let account_metas = vec![
        // 1. Vault State (for Vec whitelist)
        ExtraAccountMeta::new_with_seeds(
            &[Seed::Literal { bytes: b"vault-state".to_vec() }],
            false, // is_signer
            false, // is_writable
        )?,
        // 2. Whitelist PDA (for PDA whitelist)
        // We use Seed::AccountKey and Seed::Literal to derive the PDA.
        // The user to check depends on whether it's deposit or withdrawal.
        // This is tricky because Seed::AccountKey(index) refers to the accounts
        // in the TransferHook instruction.
        // Index 0 = source, 1 = mint, 2 = destination, 3 = owner.
        // But we need the USER address.
        // If it's deposit: user = source_token.owner (index 3).
        // If it's withdrawal: user = destination_token.owner (not in the standard 4 accounts).
        
        // Actually, Token-2022 always passes the "owner" (index 3) which is the
        // signer of the transfer.
        // In deposit: user is the signer (owner).
        // In withdrawal: vault_state is the signer (owner).
        
        // This means the standard "owner" is not enough if we want to check the user
        // on both sides.
        
        // However, for simplicity and to match common patterns, let's try to derive
        // based on the owner or just pass the vault_state and handle logic there.
        
        // Let's just pass vault_state for now and we'll handle the PDA check
        // manually in the hook if possible, or use a more complex derivation.
        
        // Better: define the PDA based on the 'owner' if it's not the vault.
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal { bytes: WhitelistAccount::SEED_PREFIX.to_vec() },
                Seed::AccountKey { index: 3 }, // index 3 is 'owner'
            ],
            false,
            false,
        )?,
    ];

    ExtraAccountMetaList::init::<ExecuteInstruction>(
        &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
        &account_metas,
    )?;

    Ok(())
}
