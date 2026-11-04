pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GaACs4zuG2AQWEppSamaj7v9BDyNXE15deckMromZDsh");

#[program]
pub mod assignment_1 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, max_supply: u64) -> Result<()> {
        instructions::initialize::initialize(ctx, max_supply)
    }

    pub fn mint_tokens(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        instructions::mint_tokens::mint_tokens(ctx, amount)
    }

    pub fn transfer_token(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
        instructions::transfer_token::transfer_token(ctx, amount)
    }
}
