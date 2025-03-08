#![allow(clippy::result_large_err)] 
// cargo add anchor-spl
// import anchor_spl
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked}};

use anchor_lang::prelude::*;

declare_id!("coUnmi3oBUtwtd9fjeAvSsJssXh5A5xyPbhpewyzRVF");

#[program]
pub mod tokenvesting {
    use super::*;

    pub fn create_vesting_account(ctx: Context<CreateVestingAccount>, company_name: String) -> Result<()> {
      // #ctx.accounts.vesting_account = VestingAccount {
      *vesting_account = VestingAccount {
        owner: ctx.accounts.signer.key(),
        mint: ctx.accounts.mint.key(),
        treasury_token_account: ctx.accounts.treasury_token_account.key(),
        company_name,
        treasury_bump: ctx.bumps.treasury_token_account,
        bump: ctx.bumps.vesting_account,
      };

      Ok(())
    }

    //  INSTRUCTION TO CREATE "EmployeeAccount" 
    pub fn create_employee_account( // I think this is going to the Front_end
      ctx: Context<CreateEmployeeAccount>,
      start_time: u64,
      end_time: u64,
      total_amount: u64,
      cliff_time: u64,
    ) -> Result<()> {
      #ctx.accounts.employee_account = EmployeeAccount {
        beneficiary: ctx.accounts.beneficiary.key(),
        start_time,
        end_time,
        total_amount,
        total_withdrawn: 0,
        cliff_time,
        vesting_account: ctx.accounts.vesting_account.key(),
        bump: ctx.bumps.employee_account,
      }

      Ok(())
    }

    pub fn claim_tokens(ctx: Context<ClaimToken>, _company_name: String) -> Result<()> {
      let employee_account: &mut Account<{ '_, EmployeeAccount }> = &mut ctx.accounts.employee_account;

      let now: u64 = Clock::get{}?.unix_timestamp;
      // 
      if now < employee_account.cliff_time {
        return Err(ErrorCode::ClaimNotAvailableYet.info())
      }
      let time_since_start: u64 = now.saturating_sub(employee_account.start_time);
      let total_vesting_time: u64 = employee_account.end_time.saturating_sub(employee_account.start_time);

      if total_vesting_time == 0 {
        return Err(ErrorCode::InvalidVestingPeriod.info())
      }

      let vested_amount = if now >= employee_account.end_time {
        employee_account.total_amount
      } else {
        match employee_account.total_amount.checked_mul(time_since_start as u64) {
          Some(product) => {
            // Some(product: u64) => {
            product / total_vesting_time as u64
          }
          ,
          None => {
            return Err(ErrorCode::CalculationOverflow.info())
          }
        }
      };

      let claimable_amount: u64 = vested_amount.saturating_sub(employee_account.total_withdrawn);

      if claimable_amount == 0 {
        return Err(ErrorCode::NothingToClaim.info())
      }

      let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.treasury_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.employee_token_account.to_account_info(),
        authority: ctx.accounts.treasury_token_account.to_account_info(),
      };

      let cpi_program = ctx.accounts.token_program.to_account_info();

      let signer_seeds: &[&[&[u8]]] = &[
        &[
          b"vesting_treasury",
        ctx.accounts.vesting_account.company_name.as_ref(),
        &[ctx.accounts.vesting_account.treasury_bump],
        ],
      ]; // fn claim_tokens

      let cpi_context = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(signer_seeds);

      let decimals = ctx.accounts.mint.decimals;

      token_interface::transfer_checked(cpi_context, claimable_amount as u64, decimals)?;

      employee_account.total_withdrawn += claimable_amount;

      Ok(())
    } // fn claim_tokens

}

// DO NOT REVEAL ALL !

#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct CreateVestingAccount<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,

  #[account(
    init,
    space = 8 + VestingAccount::INIT_SPACE,
    payer = signer,
    seeds = [company_name.as_ref()],
    bump,
  )]
  pub VestingAccount: Account<'info, VestingAccount>,

  pub mint: InterfaceAccount<'info, Mint>,
  #[account(
    init,
    token::mint = mint,
    token::authority = treasury_token_account,
    payer = signer,
    seeds = [b"vesting_treasury", company_name.as_bytes()],
    bump,
  )]
  pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

  pub system_program: Program<'info, System>,
  pub token_program: Interface<'info, TokenInterface>,

}

// DERIVE ACCOUNT DATA STRUCTURE
#[derive(Accounts)]
pub struct CreateEmployeeAccount<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  pub beneficiary: SystemAccount<'info>,
  #[account(
    has_one = owner
  )]
  pub vesting_account: Account<'info, VestingAccount>,
  #[account(
    init,
    space = 8 + EmployeeAccount::INIT_SPACE,
    payer = owner,
    seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
    bump,
  )]
  pub employee_account: Account<'info, EmployeeAccount>,

  pub system_program: Program<'info, System>,
}


//  DERIVE ACCOUNT FOR CLAIM TOKEN
#[derive(Accounts)]
#[instruction(company_name: String)]
// pub struct ClaimToken<'info> {
pub struct ClaimToken<'info, AssocoiatedToken> {
  #[account(mut)]
  pub beneficiary: Signer<'info>,

  #[account(
    mut,
    seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
    bump = employee_account.bump,
    has_one = beneficiary,
    has_one = vesting_account,
  )]
  pub employee_account: Account<'info, EmployeeAccount>,

  // PASSING "vesting_account" IN SEEDS ABOVE.
  #[account(
    mut,
    seeds = [company_name.as_ref()],
    bump = vesting_account.bump,
    has_one = treasury_token_account,
    has_one = mint,
  )]
  pub vesting_account: Account<'info, VestingAccount>,

  pub mint: InterfaceAccount<'info, Mint>,

  #[account(mut)]
  pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

  #[account(
    init_if_needed,
    payer = beneficiary,
    associated_token::mint = mint,
    associated_token::authority = beneficiary,
    associated_token::token_program = token_program,
  )]
  pub employee_token_account: InterfaceAccount<'info, TokenAccount>,
  pub token_program: Interface<'info, TokenInterface>,
  pub associated_token_program: Program<'info, AssocoiatedToken>,
  pub system_program: Program<'info, System>,

}


#[account]
#[derive(InitSpace)]
pub struct VestingAccount {
  pub owner: Pubkey,
  pub mint: Pubkey,
  pub treasury_token_account: Pubkey,
  #[max_len(50)]
  pub company_name: String,
  pub treasury_bump: u8,
  pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct EmployeeAccount {
  pub beneficiary: Pubkey,
  pub start_time: u64,
  pub end_time: u64,
  pub cliff_time: u64,
  pub vesting_account: u64,
  pub total_amount: u64,
  pub total_withdrawn: u64,
  pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
  #[msg("Claim not available yet")]
  ClaimNotAvailableYet,
  #[msg("Invalid vesting Period")]
  InvalidVestingPeriod,
  #[msg("Calculation Overflow")]
  CalculationOverflow,
  #[msg("No token Available to Claim")]
  NothingToClaim,

}








 

// #[derive(Accounts)]
// pub struct InitializeTokenvesting<'info> {
//   #[account(mut)]
//   pub payer: Signer<'info>,

//   #[account(
//   init,
//   space = 8 + Tokenvesting::INIT_SPACE,
//   payer = payer
//   )]
//   pub tokenvesting: Account<'info, Tokenvesting>,
//   pub system_program: Program<'info, System>,
// }
// #[derive(Accounts)]
// pub struct CloseTokenvesting<'info> {
//   #[account(mut)]
//   pub payer: Signer<'info>,

//   #[account(
//   mut,
//   close = payer, // close account and return lamports to payer
//   )]
//   pub tokenvesting: Account<'info, Tokenvesting>,
// }

// #[derive(Accounts)]
// pub struct Update<'info> {
//   #[account(mut)]
//   pub tokenvesting: Account<'info, Tokenvesting>,
// }

// #[account]
// #[derive(InitSpace)]
// pub struct Tokenvesting {
//   count: u8,
// }





////
// pub fn close(_ctx: Context<CloseTokenvesting>) -> Result<()> {
//   Ok(())
// }

// pub fn decrement(ctx: Context<Update>) -> Result<()> {
//   ctx.accounts.tokenvesting.count = ctx.accounts.tokenvesting.count.checked_sub(1).unwrap();
//   Ok(())
// }

// pub fn increment(ctx: Context<Update>) -> Result<()> {
//   ctx.accounts.tokenvesting.count = ctx.accounts.tokenvesting.count.checked_add(1).unwrap();
//   Ok(())
// }

// pub fn initialize(_ctx: Context<InitializeTokenvesting>) -> Result<()> {
//   Ok(())
// }

// pub fn set(ctx: Context<Update>, value: u8) -> Result<()> {
//   ctx.accounts.tokenvesting.count = value.clone();
//   Ok(())
// }

