# SPL Token Program (Anchor)

A Solana program built with Anchor 1.0 that creates its own SPL token. Anyone can mint, up to a fixed supply cap, and holders can transfer to any wallet.

## How it works

- **Config PDA** (`seeds = [b"config"]`) stores the mint address, the max supply and the bump.
- The **config PDA is the mint authority**. A user's wallet can't sign for the program, so the program signs `mint_to` itself with the PDA seeds. This is what lets any user call `mint_tokens` while the program enforces the cap.
- The **cap is checked against the live mint supply** before every mint, so it applies to all users combined.
- Token accounts are **associated token accounts**, created on the fly with `init_if_needed` when they don't exist.

## Instructions

| Instruction | Who can call | What it does |
|---|---|---|
| `initialize(max_supply)` | anyone, once | Creates the config PDA and the mint (6 decimals) with the config PDA as mint authority |
| `mint_tokens(amount)` | anyone | Mints `amount` to the caller's ATA, rejected if it would pass `max_supply` |
| `transfer_token(amount)` | token holder | Moves `amount` from the sender's ATA to the recipient's ATA (created if missing) |

`max_supply` and `amount` are in base units. With 6 decimals, a cap of 20 tokens is `20_000_000`.

## Accounts

```rust
pub struct TokenConfig {
    pub mint: Pubkey,
    pub max_supply: u64,
    pub bump: u8,
}
```

## Build and test

Requirements: Rust, Solana CLI, Anchor 1.0.

```bash
anchor build
cargo test        # LiteSVM tests in programs/assignment_1/tests/
```

Tests cover initialize, minting within the cap, minting over the cap (rejected), transfers (including creating the recipient's ATA), and transferring more than the balance (rejected).

## Deploy to devnet

```bash
solana config set --url devnet
anchor keys sync
anchor build
anchor program deploy --provider.cluster devnet
```

- Program ID: `<PROGRAM_ID>`
- Explorer: `https://explorer.solana.com/address/GaACs4zuG2AQWEppSamaj7v9BDyNXE15deckMromZDsh?cluster=devnet`

## Run the demo script

```bash
yarn add @anchor-lang/core @solana/web3.js@1 @solana/spl-token --ignore-engines
yarn add -D tsx --ignore-engines

ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
ANCHOR_WALLET=$HOME/.config/solana/id.json \
npx tsx scripts/invoke.ts
```

The script initializes (once), mints 5 tokens, tries to mint 100 (rejected by the cap), then transfers 2 tokens to a fresh wallet and prints the balances and Explorer links.

## What I learned

- A PDA can be the mint authority, and the program signs for it with its seeds.
- The ATA address is a PDA derived from the wallet, the token program and the mint.
- A transfer moves balances between token accounts and never changes total supply.
- Account constraints (`has_one`, `associated_token::`, signers) reject fake accounts before the handler runs, and the handler enforces the cap with `require!`.

## Links

- Write-up: `https://x.com/SegFault_Dev/status/2118262854856118344`
