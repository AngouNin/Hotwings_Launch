use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Insufficient funds in the sender's account.")]
    InsufficientFunds,

    #[msg("Max hold amount exceeded for the receiver's account.")]
    MaxHoldExceeded,
}