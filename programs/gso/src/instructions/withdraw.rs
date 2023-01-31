use anchor_spl::token::{Token, TokenAccount};
use staking_options::program::StakingOptions as StakingOptionsProgram;

pub use crate::common::*;
pub use crate::*;

pub fn withdraw(ctx: Context<GSOWithdraw>) -> Result<()> {
    msg!("GSO Withdraw");

    msg!("CPI into SO");
    let so_withdraw_accounts = staking_options::cpi::accounts::Withdraw {
        authority: ctx.accounts.so_authority.to_account_info(),
        state: ctx.accounts.so_state.to_account_info(),
        base_vault: ctx.accounts.so_base_vault.to_account_info(),
        base_account: ctx.accounts.user_base_account.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_program_withdraw = ctx.accounts.staking_options_program.to_account_info();

    staking_options::cpi::withdraw(CpiContext::new_with_signer(
        cpi_program_withdraw,
        so_withdraw_accounts,
        &[&[
            "gso".as_bytes(),
            &ctx.accounts.gso_state.key().to_bytes(),
            &[ctx.accounts.gso_state.so_authority_bump],
        ]],
    ))?;

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct GSOWithdraw<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [GSO_STATE_SEED, &gso_state.period_num.to_be_bytes(), &gso_state.project_name.as_bytes()],
        bump = gso_state.gso_state_bump,
        constraint = gso_state.authority.key() == authority.key())]
    pub gso_state: Box<Account<'info, GSOState>>,

    /// CHECK: Not dangerous. Just an AccountInfo for signing.
    #[account(mut,
        seeds = [SO_AUTHORITY_SEED, &gso_state.key().to_bytes()],
        bump = gso_state.so_authority_bump
    )]
    pub so_authority: AccountInfo<'info>,

    /// Where the tokens are going.
    #[account(mut)]
    pub user_base_account: Box<Account<'info, TokenAccount>>,

    /// Where the tokens are coming from.
    #[account(mut)]
    pub so_base_vault: Box<Account<'info, TokenAccount>>,

    /// Seeds are verified in the CPI. Verified that this is the correct
    /// so_state here.
    #[account(mut, constraint = so_state.key() == gso_state.staking_options_state)]
    pub so_state: Box<Account<'info, staking_options::State>>,

    pub staking_options_program: Program<'info, StakingOptionsProgram>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
