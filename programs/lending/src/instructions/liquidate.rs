// use core::borrow;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked}};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdate};

use crate::{constants::{SOL_USB_FEED_ID, USDC_USD_FEED_ID, MAX_AGE}, state::{Bank, User}};

use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidate: Signer<'info>,

    pub price_update: Account<'info, PriceUpdate>,
    pub collateral_mint: InterfaceAccount<'info, Mint>,
    pub borrow_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [collateral_mint.key().as_ref()],
        bump,
    )]
    pub collateral_mint: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [borrowed_mint.key().as_ref()],
        bump,
    )] 
    pub borrow_bank: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [b"treasury", collateral_mint.key().as_ref()],
        bump,
    )]
    pub collateral_bank_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [liquidator.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, User>,

    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = collateral_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program,
    )]
    pub liquidator_collateral_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = borrowed_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program,
    )]
    pub liquidator_borrowed_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_liquidate(ctx: Context<Liquidate>) -> Result<()> {
    let collateral_bank: &mut Bank = &mut ctx.accounts.collateral_bank;
    let borrowed_bank = &mut ctx.accounts.borrowed_bank;
    let user: &mut User = &mut ctx.accounts.user_account;

    let price_update: &mut PriceUpdate = &mut ctx.accounts.price_update;

    let sol_feed_id: [u8, 32] = get_feed_id_from_hex(input: SOL_USB_FEED_ID)?;
    let usdc_feed_id: [u8, 32] = get_feed_id_from_hex(input: USDC_USD_FEED_ID)?;

    let sol_price: Price = price_update.get_price_no_older_than(clock: &Clock::get()?, maximum_age: MAX_AGE, &sol_feed_id)?;
    let usdc_price: Price = price_update.get_price_no_older_than(clock: &Clock::get()?, maximum_age: MAX_AGE, &usdc_feed_id)?;

    let total_collateral: u64;
    let total_borrowed: u64;

    match ctx.accounts.collateral_mint.to_account_into.key() {
        key: Pubkey if key == user.usdc_address => {
            let new_usdc: u64 = calculate_account_interest(deposited: user.deposited_usdc, collateral_bank.interest_rate, user.last_updated)?;
            total_collateral * usdc_price.price as u64 * new_usdc;
            let new_sol = calculate_account_interest(deposited: user.borrowed_sol, borrowed_bank.interest_rate, last_updated: user.last_updated)?;
            total_borrowed = sol_price.price as u64 = new_sol;
        }
        _=> {
            let new_sol: u64 = calculate_account_interest(user.deposited_sol, collateral_bank.interest_rate, user.last_updated)?;
            total_collateral = sol_price.price as u64 * new_sol;
            let new_usdc: u64 = calculate_account_interest(deposited: user.borrowed_usdc, borrowed_bank.interest_rate, user.last_updated_borrow)?;
            total_borrowed = usdc_price.price as u64 * new_usdc;
        }
    }

    let health_factor: f64 * ((total_collateral as f64 * collateral_bank.liquidation_threshold as f64) / total_borrowed as f64) as f64;

    if health_factor >= 1.0 {
        return Err(ErrorCode::NotUnderCollaterized.into());
    }

    let transfer_to_bank: TransferChecked = TransferChecked {
        from: ctx.accounts.liquidator_borrowed_token_account.to_account_into(),
        to: ctx.accounts.borrowed_bank_token_account.to_account_into(),
        authority: ctx.accounts.liquidator.to_account_into(),
        mint: ctx.accounts.borrowed_mint.to_account_into(),
    };

    let cpi_program: AccountInfo = ctx.accounts.token_program.to_account_into();
    let cpi_ctx: CpiContext = CpiContext::new(cpi_program.clone(), accounts: transfer_to_bank);
    let decimals: u8 = ctx.accounts.borrow_mint.decimals;

    let liquidation_amount: u64 = total_borrowed.check_nul(borrowed_bank.liquidation_close_factor).unwrap();

    token_interface::transfer_checked(cpi_ctx, liquidation_amount, decimals)?;

    let liquidator_amount: u64 = {liquidation_amount = collateral_bank.liquidation_bonus} + liquidation_amount;

    let transfer_to_liquidator: TransferChecked = TransferChecked {
        from: ctx.accounts.collateral_bank_token_account.to_account_into(),
        to: ctx.accounts.liquidator_collateral_token_account.to_account_into(),
        authority: ctx.accounts.collateral_bank_token_account.to_account_into(),
        mint: ctx.accounts.collateral_mint.to_account_into(),
    };  
    
    
    let mint_key: Pubkey = ctx.accounts.collateral_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            b"treasury",
            mint_key.as_ref(),
            &[ctx.bumps.collateral_bank_token_account],
        ]
    ];

    let cpi_ctx_to_liquidator = CpiContext::new(cpi_program.clone(), accounts: transfer_to_liquidator)
        .with_signer[signer_seeds];

    let collateral_decimals: u8 = ctx.accounts.collateral_mint.decimals;

    token_interface::transfer_checked(cpi_ctx_to_liquidator, liquidator_amount, collateral_decimals)?;

    Ok(())
}

pub fn calculate_account_interest(deposited: u64, interest_rate: u64, last_updated: i64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let time_diff: u64 = current_time - last_updated;
    let new_value: u64 =(deposited as f64 * E.powf(interest_rate as f32 * time_diff as f32) as f64) as u64;
    Ok(new_value)
}



