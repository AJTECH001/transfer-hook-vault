#![allow(unexpected_cfgs)]

use anchor_lang::{
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    },
    InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use spl_associated_token_account_interface::{
    address::get_associated_token_address_with_program_id,
    instruction::create_associated_token_account,
};
use std::str::FromStr;
const TOKEN_2022_ID_STR: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

fn token_2022_id() -> Pubkey {
    Pubkey::from_str(TOKEN_2022_ID_STR).unwrap()
}
use transfer_hook_vault as program;

// ─── helpers ──────────────────────────────────────────────────────────────────

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn program_id() -> Pubkey {
    program::id()
}

fn vault_state_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"vault-state"], &program_id()).0
}

fn mint_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"vault-mint"], &program_id()).0
}

fn vault_token_account() -> Pubkey {
    get_associated_token_address_with_program_id(&vault_state_pda(), &mint_pda(), &token_2022_id())
}

fn extra_meta_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"extra-account-metas", mint_pda().as_ref()], &program_id()).0
}

fn whitelist_pda(user: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"whitelist", user.as_ref()], &program_id()).0
}

/// Boots LiteSVM, airdrops to admin, loads the compiled program .so.
fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let admin = Keypair::new();
    // Use the actual compiled binary from target/deploy
    let program_path = std::path::Path::new("target/deploy/transfer_hook_vault.so");
    if program_path.exists() {
        svm.add_program(
            program_id(),
            &std::fs::read(program_path).unwrap(),
        );
    } else {
        // Try crate-relative path if run from programs/transfer-hook-vault
        let program_path_crate = std::path::Path::new("../../target/deploy/transfer_hook_vault.so");
        if program_path_crate.exists() {
            svm.add_program(
                program_id(),
                &std::fs::read(program_path_crate).unwrap(),
            );
        } else {
            // Fallback to include_bytes
             let _ = svm.add_program(
                  program_id(),
                  include_bytes!("../../../target/deploy/transfer_hook_vault.so").as_ref(),
              );
        }
    }
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    (svm, admin)
}

/// Calls initialize_vault and initialize_extra_account_meta_list.
fn initialize_vault(svm: &mut LiteSVM, admin: &Keypair, initial_supply: u64) -> Pubkey {
    let vault_state = vault_state_pda();
    let mint = mint_pda();
    let vault_ta = vault_token_account();

    let init_ix = Instruction::new_with_bytes(
        program_id(),
        program::instruction::InitializeVault { initial_supply }.data().as_slice(),
        program::accounts::InitializeVault {
            admin: admin.pubkey(),
            vault_state,
            mint,
            vault_token_account: vault_ta,
            token_program: token_2022_id(),
            associated_token_program: spl_associated_token_account_interface::program::ID,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
    );

    let extra_meta_ix = Instruction::new_with_bytes(
        program_id(),
        program::instruction::InitializeExtraAccountMetaList {}.data().as_slice(),
        program::accounts::InitializeExtraAccountMetaList {
            payer: admin.pubkey(),
            extra_account_meta_list: extra_meta_pda(),
            mint,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
    );

    send(svm, &[init_ix, extra_meta_ix], admin, &[admin]).expect("initialize_vault + extra_meta failed");
    vault_state
}

/// Calls add_to_whitelist_vec.
fn add_to_whitelist(
    svm: &mut LiteSVM,
    admin: &Keypair,
    user: Pubkey,
) -> litesvm::types::TransactionResult {
    let ix = Instruction::new_with_bytes(
        program_id(),
        program::instruction::AddToWhitelistVec { user, amount: 0 }.data().as_slice(),
        program::accounts::AddToWhitelistVec {
            admin: admin.pubkey(),
            vault_state: vault_state_pda(),
        }
        .to_account_metas(None),
    );
    send(svm, &[ix], admin, &[admin])
}

/// Calls add_to_whitelist_pda.
fn add_to_whitelist_pda(
    svm: &mut LiteSVM,
    admin: &Keypair,
    user: Pubkey,
) -> litesvm::types::TransactionResult {
    let ix = Instruction::new_with_bytes(
        program_id(),
        program::instruction::AddToWhitelistPda { user, amount: 0 }.data().as_slice(),
        program::accounts::AddToWhitelistPDA {
            admin: admin.pubkey(),
            vault_state: vault_state_pda(),
            whitelist_pda: whitelist_pda(user),
            system_program: system_program::id(),
        }
        .to_account_metas(None),
    );
    send(svm, &[ix], admin, &[admin])
}

/// Calls remove_from_whitelist_vec.
fn remove_from_whitelist(
    svm: &mut LiteSVM,
    admin: &Keypair,
    user: Pubkey,
) -> litesvm::types::TransactionResult {
    let ix = Instruction::new_with_bytes(
        program_id(),
        program::instruction::RemoveFromWhitelistVec { user }.data().as_slice(),
        program::accounts::RemoveFromWhitelistVec {
            admin: admin.pubkey(),
            vault_state: vault_state_pda(),
        }
        .to_account_metas(None),
    );
    send(svm, &[ix], admin, &[admin])
}

/// Calls deposit(amount) for a given user.
fn deposit(
    svm: &mut LiteSVM,
    user: &Keypair,
    user_token_account: Pubkey,
    amount: u64,
) -> litesvm::types::TransactionResult {
    let mut accounts = program::accounts::Deposit {
        user: user.pubkey(),
        vault_state: vault_state_pda(),
        mint: mint_pda(),
        user_token_account,
        vault_token_account: vault_token_account(),
        extra_account_meta_list: extra_meta_pda(),
        whitelist_pda: whitelist_pda(user.pubkey()),
        transfer_hook_program: program_id(),
        token_program: token_2022_id(),
        associated_token_program: spl_associated_token_account_interface::program::ID,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    // Add remaining accounts required by the transfer hook (as defined in ExtraAccountMetaList)
    // 1. ExtraAccountMetaList PDA
    accounts.push(AccountMeta::new_readonly(extra_meta_pda(), false));
    // 2. Vault State
    accounts.push(AccountMeta::new_readonly(vault_state_pda(), false));
    // 3. Whitelist PDA
    accounts.push(AccountMeta::new_readonly(whitelist_pda(user.pubkey()), false));

    let ix = Instruction::new_with_bytes(
        program_id(),
        program::instruction::Deposit { amount }.data().as_slice(),
        accounts,
    );
    // In LiteSVM/Solana, we need all accounts involved in the CPI to be in the transaction
    // but they don't all need to be in the instruction metas if they are resolved on-chain.
    // However, since we are building the instruction manually, we match what the program expects.
    send(svm, &[ix], user, &[user])
}

/// Calls withdraw(amount) for a given user.
fn withdraw(
    svm: &mut LiteSVM,
    user: &Keypair,
    user_token_account: Pubkey,
    amount: u64,
) -> litesvm::types::TransactionResult {
    let mut accounts = program::accounts::Withdraw {
        user: user.pubkey(),
        vault_state: vault_state_pda(),
        mint: mint_pda(),
        vault_token_account: vault_token_account(),
        user_token_account,
        extra_account_meta_list: extra_meta_pda(),
        whitelist_pda: whitelist_pda(user.pubkey()),
        transfer_hook_program: program_id(),
        token_program: token_2022_id(),
        associated_token_program: spl_associated_token_account_interface::program::ID,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    // Add remaining accounts required by the transfer hook
    // 1. ExtraAccountMetaList PDA
    accounts.push(AccountMeta::new_readonly(extra_meta_pda(), false));
    // 2. Vault State
    accounts.push(AccountMeta::new_readonly(vault_state_pda(), false));
    // 3. Whitelist PDA
    accounts.push(AccountMeta::new_readonly(whitelist_pda(user.pubkey()), false));

    let ix = Instruction::new_with_bytes(
        program_id(),
        program::instruction::Withdraw { amount }.data().as_slice(),
        accounts,
    );
    send(svm, &[ix], user, &[user])
}

/// Creates a Token-2022 ATA for a user and returns its address.
fn create_user_ata(svm: &mut LiteSVM, payer: &Keypair, owner: Pubkey) -> Pubkey {
    let mint = mint_pda();
    let ata =
        get_associated_token_address_with_program_id(&owner, &mint, &token_2022_id());
    let ix = create_associated_token_account(
        &payer.pubkey(),
        &owner,
        &mint,
        &token_2022_id(),
    );
    send(svm, &[ix], payer, &[payer]).expect("create_user_ata failed");
    ata
}

/// Reads the token balance of any token account from LiteSVM state.
fn token_balance(svm: &LiteSVM, token_account: &Pubkey) -> u64 {
    use spl_token_2022_interface::state::Account;
    use anchor_lang::solana_program::program_pack::Pack;
    let data = svm.get_account(token_account).unwrap().data;
    // Token-2022 accounts have a base state + optional extensions.
    // The first 165 bytes are the base SPL token account (same as Token v1).
    Account::unpack(&data[..Account::LEN]).unwrap().amount
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[test]
fn test_initialize_vault_creates_accounts() {
    let (mut svm, admin) = setup();
    let initial_supply = 1_000_000 * 10u64.pow(6);

    initialize_vault(&mut svm, &admin, initial_supply);

    // VaultState PDA must exist.
    assert!(
        svm.get_account(&vault_state_pda()).is_some(),
        "vault_state PDA should exist"
    );

    // Mint PDA must exist.
    assert!(
        svm.get_account(&mint_pda()).is_some(),
        "mint PDA should exist"
    );

    // Vault token account must hold the initial supply.
    assert_eq!(
        token_balance(&svm, &vault_token_account()),
        initial_supply,
        "vault should hold the initial supply"
    );
}

#[test]
fn test_whitelisted_user_can_withdraw() {
    let (mut svm, admin) = setup();
    let user = Keypair::new();
    let supply = 1_000_000 * 10u64.pow(6);

    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();
    initialize_vault(&mut svm, &admin, supply);
    add_to_whitelist(&mut svm, &admin, user.pubkey()).unwrap();

    let user_ata = create_user_ata(&mut svm, &admin, user.pubkey());

    let withdraw_amount = 100 * 10u64.pow(6);
    withdraw(&mut svm, &user, user_ata, withdraw_amount).expect("whitelisted user should withdraw");

    assert_eq!(token_balance(&svm, &user_ata), withdraw_amount);
    assert_eq!(
        token_balance(&svm, &vault_token_account()),
        supply - withdraw_amount
    );
}

#[test]
fn test_non_whitelisted_user_cannot_withdraw() {
    let (mut svm, admin) = setup();
    let user = Keypair::new();

    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();
    initialize_vault(&mut svm, &admin, 1_000_000 * 10u64.pow(6));

    // Do NOT add user to whitelist.
    let user_ata = create_user_ata(&mut svm, &admin, user.pubkey());

    let res = withdraw(&mut svm, &user, user_ata, 100 * 10u64.pow(6));
    assert!(res.is_err(), "non-whitelisted user must not withdraw");
}

#[test]
fn test_whitelisted_user_can_deposit() {
    let (mut svm, admin) = setup();
    let user = Keypair::new();
    let supply = 1_000_000 * 10u64.pow(6);
    let deposit_amount = 200 * 10u64.pow(6);

    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();
    initialize_vault(&mut svm, &admin, supply);
    add_to_whitelist(&mut svm, &admin, user.pubkey()).unwrap();

    let user_ata = create_user_ata(&mut svm, &admin, user.pubkey());

    // Give the user some tokens first (withdraw from vault).
    withdraw(&mut svm, &user, user_ata, deposit_amount).unwrap();

    let vault_balance_before = token_balance(&svm, &vault_token_account());
    let user_balance_before = token_balance(&svm, &user_ata);

    // Now deposit them back.
    deposit(&mut svm, &user, user_ata, deposit_amount).expect("whitelisted user should deposit");

    assert_eq!(
        token_balance(&svm, &vault_token_account()),
        vault_balance_before + deposit_amount
    );
    assert_eq!(
        token_balance(&svm, &user_ata),
        user_balance_before - deposit_amount
    );
}

#[test]
fn test_non_whitelisted_user_cannot_deposit() {
    let (mut svm, admin) = setup();
    let user = Keypair::new();
    let whitelisted = Keypair::new();
    let supply = 1_000_000 * 10u64.pow(6);

    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();
    svm.airdrop(&whitelisted.pubkey(), 1_000_000_000).unwrap();
    initialize_vault(&mut svm, &admin, supply);

    // Only whitelist the helper, not the attacker.
    add_to_whitelist(&mut svm, &admin, whitelisted.pubkey()).unwrap();

    let whitelisted_ata = create_user_ata(&mut svm, &admin, whitelisted.pubkey());
    let user_ata = create_user_ata(&mut svm, &admin, user.pubkey());

    // Give the whitelisted user some tokens.
    withdraw(&mut svm, &whitelisted, whitelisted_ata, 100 * 10u64.pow(6)).unwrap();

    // Try to deposit FROM the non-whitelisted user's ATA.
    // (In a real attack, they'd have obtained tokens somehow.)
    // Our check: vault_state.whitelist.contains(&user.key()) — fails.
    let res = deposit(&mut svm, &user, user_ata, 1);
    assert!(res.is_err(), "non-whitelisted user must not deposit");
}

#[test]
fn test_remove_then_readd() {
    let (mut svm, admin) = setup();
    let user = Keypair::new();

    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();
    initialize_vault(&mut svm, &admin, 1_000_000 * 10u64.pow(6));
    add_to_whitelist(&mut svm, &admin, user.pubkey()).unwrap();

    let user_ata = create_user_ata(&mut svm, &admin, user.pubkey());

    // Can withdraw while whitelisted.
    assert!(
        withdraw(&mut svm, &user, user_ata, 10 * 10u64.pow(6)).is_ok(),
        "should succeed while whitelisted"
    );

    // Remove from whitelist.
    remove_from_whitelist(&mut svm, &admin, user.pubkey()).unwrap();

    // Now blocked.
    assert!(
        withdraw(&mut svm, &user, user_ata, 10 * 10u64.pow(6)).is_err(),
        "should fail after removal"
    );

    // Re-add and try again.
    add_to_whitelist(&mut svm, &admin, user.pubkey()).unwrap();
    assert!(
        withdraw(&mut svm, &user, user_ata, 10 * 10u64.pow(6)).is_ok(),
        "should succeed after re-add"
    );
}

#[test]
fn test_non_admin_cannot_add_to_whitelist() {
    let (mut svm, admin) = setup();
    let attacker = Keypair::new();
    let victim = Keypair::new();

    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();
    initialize_vault(&mut svm, &admin, 1_000_000 * 10u64.pow(6));

    // Attacker tries to add themselves.
    let ix = Instruction::new_with_bytes(
        program_id(),
        &program::instruction::AddToWhitelistVec { user: attacker.pubkey(), amount: 0 }.data(),
        program::accounts::AddToWhitelistVec {
            admin: attacker.pubkey(), // attacker pretends to be admin
            vault_state: vault_state_pda(),
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &[ix], &attacker, &[&attacker]);
    assert!(res.is_err(), "non-admin must not add to whitelist");

    // victim is still not whitelisted.
    let _ = victim;
}

#[test]
fn test_vault_starts_with_correct_supply() {
    let (mut svm, admin) = setup();
    let supply = 500_000 * 10u64.pow(6);

    initialize_vault(&mut svm, &admin, supply);

    assert_eq!(
        token_balance(&svm, &vault_token_account()),
        supply,
        "vault should hold exactly the initial supply"
    );
}
