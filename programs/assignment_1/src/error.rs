use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Mint would exceed the max supply")]
    SupplyCapExceeded,
    #[msg("account balance low")]
    InsufficientBalance,
}
