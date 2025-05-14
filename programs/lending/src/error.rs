use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient Funds")]
    InsufficientFunds,
    #[msg("Requested amount is more than borrowable amount.")]
    OverBorrowableAmount,
    // ...
    #[msg("Requested amount is more than repayable amount.")]
    OverRepay,
    #[msg("User is not under collaterized, can't be liquidated")]
    NotUnderCollaterized,
}



