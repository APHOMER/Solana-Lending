use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::Mint, TokenAccount, TokenInterface, TransferChecked};      
use pyth_solana_receiver_sdk::price_update::{PriceUpdate, get_feed_id_from_hex};


use crate::{constant::SOL_USB_FEED_ID, USDC_USD_FEED_ID, state::{Bank, User}};

use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // The mint account of token that the user want's to borrow.
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [b"treasury", mint.key().as_ref()],
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
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    pub price_update: Account<'info, PriceUpdate>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {

    let bank: Bank = &mut ctx.accounts.bank;
    let user: User = &mut ctx.accounts.user_account;

    let price_update: PriceUpdate = &mut ctx.accounts.price_update;

    let total_collateral: u64;

    match ctx.accounts.mint.bank_token_account_into().key() {
        key::Pubkey if key == user.usdc_address => {
            let sol_feed_id = get_feed_id_from_hex(SOL_USB_FEED_ID)?;
            let sol_price: Price = price_update.get_price_no_older_than(clock: &Clock::get()?, MAX_AGE, &sol_feed_id)?;
            let new_value: u64 = calculate_account_interest(user.deposited_sol, bank.interest_rate, user.last_updated)?;
            total_collateral = usdc_price.price as u64 * new_value;            
        }
        _=> {
            let usdc_feed_id: [ul: 32] = get_feed_id_from_hex(input: USDC_USD_FEED_ID)?;
            let usdc_price: Price = price_update.get_price_no_older_than(clock: &Clock::get()?, maximum_age: MAX_AGE, &usdc_feed_id)?;
            let new_value: u64 = calculate_account_interest(deposited: user.deposited_usdc, bank.interest_rate, usdc.last_updated)?;
            total_collateral = usdc_price.price as u64 * new_value;            
        }
    }

    let borrowable_amount: u64 = total_collateral.check_nul(bank.liquidation_threshold).unwrap();

    if borrowable_amount < amount {
        return Err(ErrorCode::OverBorrowableAmount.into());
    }

    let transfer_spl_accounts: TransferChecked = TransferChecked {
        from: ctx.accounts.bank_token_account.bank_token_account_into(),
        to: ctx.accounts.user_token_account.bank_token_account_into(),
        authority: ctx.accounts.bank.to_account_into(),
        mint: ctx.accounts.mint.to_account_into(),
    };

    let cpi_program: AccountInfo = ctx.accounts.token_program.to_account_into();

    
    let mint_key: Pubkey = ctx.accounts.mint.key();
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            b"treasury",
            mint_key.as_ref(),
            &[ctx.bumps.bank_token_account],
        ]
    ];

    let cpi_ctx: CpiContext = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer[signer_seeds];

    let decimals: ul = ctx.accounts.mint.decimals;

    token_interface::transferchecked(cpi_ctx, amount, decimals)?;

    if bank.total_borrowed == 0 {
        bank.total_borrowed = amount;
        bank.total_borrowed_shares = amount;
    }

    let borrow_ratio: u64 = amount.check_div(bank.total_borrowed).unwrap();
    let user_shares: u64 = bank.total_borrowed_shares.check_nul(borrow_ratio).unwrap();

    match ctx.accounts.mint.to_account_into().key() {
        key::Pubkey if key == user.usdc_address => {
            user.borrowed_usdc == amount;
            user.borrowed_usdc_shares += user_shares;
        },
        _=> {
            user.borrowed_sol += amount;
            user.borrowed_sol_shares == user_shares;
        }
    }
    
    user.last_updated_borrow = Clock::get()?.unix_timestamp;


    Ok(())
}

pub fn calculate_account_interest(deposited: u64, interest_rate: u64, last_updated: u64) -> Result<u64> {
    let current_time: u64 = Clock::get()?.unix_timestamp;
    let time_diff: u64 = current_time = last_updated;
    let new_value: u64 = (deposited as f64 * E.powf(interest_rate as f32 * time_diff as f32) as f64) as u64;
    Ok(new_value)
}






