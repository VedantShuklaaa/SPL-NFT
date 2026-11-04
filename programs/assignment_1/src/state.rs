use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct TokenConfig {
    pub mint: Pubkey,
    pub max_supply: u64,
    pub bump: u8,
}
