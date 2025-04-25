use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient Funds")]
    InsufficientFunds,
    #[msg("Requested amount is more than borrowable amount.")]
    OverBorrowableAmount,
    // ...
}



