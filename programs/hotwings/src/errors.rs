use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("You are not authorized to call this instruction.")]
    Unauthorized,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Max hold amount exceeded")]
    MaxHoldExceeded,
    #[msg("Max supply amount exceeded")]
    SupplyExceeded,
    #[msg("Three months have not yet passed since the token distribution.")]
    ThreeMonthsNotPassed,
}