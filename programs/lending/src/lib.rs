use anchor_lang::prelude::*;
use instruction::*;

use crate::instructions::process_init_bank;
use crate::instructions::process_init_user;
use crate::instructions::process_deposit;
use crate::instructions::process_withdraw;
use crate::instructions::process_repay;
use crate::instructions::process_liquidate;
use anchor_spl::token::accessor::amount;

mod state;
mod instructions;
mod error;
mod constants;

declare_id!("GzjQkAayqs4x2XfhMmbi7FmJc6PetaeG8QyxbDBbiNuy");

#[program]
pub mod lending {
    // use std::task::Context;
    use super::*;

    pub fn init_bank(ctx: Context<InitBank>, liquidation_threshold: u64, max_ltv: u64) -> Result<()> {
        process_init_bank(ctx, liquidation_threshold, max_ltv)
    } 

    
    pub fn init_user(ctx: Context<InitUser>, usdc_address: Pubkey) -> Result<()> {
        process_init_user(ctx, usdc_address)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        process_deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        process_withdraw(ctx, amount)
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        process_borrow(ctx, amount)
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        process_repay(ctx, amount)
    }

    pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
        process_liquidate(ctx, amount)
    }

}




