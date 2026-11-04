use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    /// CHECK: only the address is used, to derive the recipient's ATA
    pub recipient: UncheckedAccount<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
		mut,
		associated_token::mint = mint,
		associated_token::authority = sender,
	)]
    pub sender_token_account: Account<'info, TokenAccount>,

    #[account(
		init_if_needed,
		payer = sender,
		associated_token::mint = mint,
		associated_token::authority = recipient,
	)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn transfer_token(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
    require!(
        ctx.accounts.sender_token_account.amount >= amount,
        ErrorCode::InsufficientBalance
    );

    let transfer_check = TransferChecked {
        from: ctx.accounts.sender_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.sender.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), transfer_check);

    transfer_checked(cpi_ctx, amount, ctx.accounts.mint.decimals)?;

    Ok(())
}
