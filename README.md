# Transfer Hook Vault

A secure Solana Anchor program demonstrating advanced Token-2022 features, including a Transfer Hook for whitelist enforcement and Metadata Pointer for token discoverability.

## Architecture

- **Vault PDA**: A single PDA (`vault-state`) that manages the protocol configuration and owns the token vault.
- **Vault Mint**: A Token-2022 mint created by the program, with the Transfer Hook extension enabled.
- **Whitelist System**: Supports two approaches for comparison:
    1. **Vec-based**: A fixed-size list stored within the `VaultState` account.
    2. **PDA-based**: Individual `WhitelistAccount` PDAs per user.
- **Transfer Hook**: An instruction that fires on every token transfer, ensuring only whitelisted users can send or receive tokens.

## Whitelist Comparison

| Feature | Vec-based (Approach 1) | PDA-based (Approach 2) |
|---------|-----------------------|-----------------------|
| **Rent Cost** | Fixed upfront (for max size) | Paid per whitelisted user |
| **Compute** | O(n) scan | O(1) lookup |
| **Scalability** | Limited (max 100 entries) | Unlimited |
| **Security** | High (contained in state) | High (isolated accounts) |
| **Flexibility** | Low (fixed size) | High (dynamic creation/deletion) |

## Security Considerations

1. **PDA Signer Validation**: All critical instructions (initialize, whitelist management) use PDA seeds and `has_one` constraints to ensure only the admin can perform sensitive actions.
2. **Transfer Hook Enforcement**: By enabling the Transfer Hook on the mint, the whitelist check cannot be bypassed even if users interact with the Token-2022 program directly.
3. **CPI Validation**: The program validates all remaining accounts passed to the transfer hook to prevent account substitution attacks.
4. **Overflow Protection**: All accounting operations use safe math or checked operations.

## Extra Extension: Metadata Pointer

The **Metadata Pointer** extension was chosen to provide a standardized way for the token to expose metadata (name, symbol, etc.). This improves the protocol's integration with the broader Solana ecosystem (wallets, explorers) while keeping the metadata management flexible.

## Project Structure

- `instructions/`: Contains the logic for each program instruction.
- `state/`: Defines the account structures and PDA seeds.
- `errors/`: Custom error codes for the protocol.
- `events/`: Events emitted for off-chain indexing.
- `hooks/`: Implementation of the Transfer Hook logic.
- `tests/`: Comprehensive tests using LiteSVM.

## How to Run Tests

1. Build the program:
   ```bash
   anchor build
   ```
2. Run the LiteSVM tests:
   ```bash
   cargo test -- --nocapture
   ```
# transfer-hook-vault
