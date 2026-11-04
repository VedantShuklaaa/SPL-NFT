use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::error::ErrorCode;
use crate::TokenConfig;

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
		seeds = [b"config"],
		bump = config.bump,
		has_one = mint,
	)]
    pub config: Account<'info, TokenConfig>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
		init_if_needed,
		payer = user,
		associated_token::mint = mint,
		associated_token::authority = user,
	)]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn mint_tokens(ctx: Context<MintToken>, amount: u64) -> Result<()> {
    let new_supply = ctx
        .accounts
        .mint
        .supply
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;

    require!(
        new_supply <= ctx.accounts.config.max_supply,
        ErrorCode::SupplyCapExceeded
    );

    let cpi_account = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.config.to_account_info(),
    };

    let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.accounts.config.bump]]];
    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_account, signer_seeds);

    mint_to(cpi_ctx, amount)?;
    Ok(())
}
