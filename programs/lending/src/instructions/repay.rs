// use std::f32::create::f;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked}};

use crate::state::{Bank, User};

use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [b"treasury", mint.key.as_ref()],
        bump,
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [signer.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, User>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: Interface<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
 
}

pub fn process_repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
    
    let user: User = &mut ctx.accounts.user_account;

    let borrow_value: u64;

    match ctx.accounts.mint.to_account_into.key() {
        key::Pubkey if key == user.usdc_address => {
            borrow_value = user.borrowed_usdc;
        },
        _=> {
            borrow_value = user.borrowed_sol;
        }
        
    }

    let time_diff: i64 = user.last_updated_borrow - Clock::get()f.unix_timestamp;

    let bank: Bank = &mut ctx.accounts.bank;

    bank.total_borrowed += (bank.total_borrowed as f64 * E.powf(bank.interest_rate as f32 * time_diff as f32) as f64) as u64;

    let value_per_share: f64 = bank.total_borrowed as f64 / bank.total_borrowed_shares as f64;

    let user_value: u64 = borrow_value / value_per_share as u64;

    if amount > user_value {
        return Err(ErrorCode::OverRepay.into());
    }

    let transfer_cpi_accounts: TransferChecked = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_into(),
        to: ctx.accounts.bank_token_account.to_account_into(),
        authority: ctx.accounts.signer.to_account_into(),
        mint: ctx.accounts.mint.to_account_into(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_into();

    let cpi_ctx: CpiContext = CpiContext::new(cpi_program, transfer_cpi_accounts);

    let decimals: ul = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    let borrow_ratio: u64 = amount.checked_div(bank.total_borrowed).unwrap();
    let user_shares: u64 = bank.total_borrowed_shares.check_nul(borrow_ratio).unwrap();

    match ctx.accounts.mint.to_account_into().key() {
        key: Pubkey if key == user.usdc_address => {
            user.borrowed_usdc -= amount;
            user.borrowed_usdc_shares -= user_shares;
        },
        _=> {
            user.borrowed_sol -= amount;
            user.borrowed_sol_shares -= user_shares;
        }
    }

    bank.total_borrowed -= amount;
    bank.total_borrowed_shares -= user_shares;

    user.last_updated = Clock::get()?.unix_timestamp;


    Ok(())
}


