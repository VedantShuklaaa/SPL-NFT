use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::TokenConfig;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + TokenConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, TokenConfig>,

    #[account(
        init, 
        payer = payer,
        mint::decimals = 6,
        mint::authority = config
    )]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, max_supply: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.mint = ctx.accounts.mint.key();
    config.max_supply = max_supply;
    config.bump = ctx.bumps.config;
    Ok(())
}
