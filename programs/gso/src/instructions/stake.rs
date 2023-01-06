use anchor_spl::token::{Mint, Token, TokenAccount};
use staking_options::program::StakingOptions as StakingOptionsProgram;

pub use crate::common::*;
pub use crate::errors::ErrorCode;
pub use crate::*;

pub fn stake(ctx: Context<GSOStake>, amount: u64) -> Result<()> {
    msg!("GSO Stake");

    msg!("Lockup tokens");
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.user_base_account.to_account_info(),
                to: ctx.accounts.base_vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info().clone(),
            },
        ),
        amount,
    )?;

    msg!("Mint xTokens");
    anchor_spl::token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.x_base_mint.to_account_info(),
                to: ctx.accounts.user_x_base_account.to_account_info(),
                authority: ctx.accounts.x_base_mint.to_account_info(),
            },
            &[&[
                X_GSO_SEED,
                &ctx.accounts.gso_state.key().to_bytes(),
                &[ctx.accounts.gso_state.x_base_mint_bump],
            ]],
        ),
        amount,
    )?;

    msg!("CPI into SO");
    // Convert to u128 to not lose precision with tokens like BONK which really
    // do need all the bits.
    let amount_128: u128 = amount as u128;
    let lockup_ratio_tokens_per_million_128: u128 =
        ctx.accounts.gso_state.lockup_ratio_tokens_per_million as u128;
    let num_staking_options_128: u128 = unwrap_int!(unwrap_int!(
        amount_128.checked_mul(lockup_ratio_tokens_per_million_128)
    )
    .checked_div(1_000_000));
    let num_staking_options: u64 = num_staking_options_128 as u64;
    let so_issue_accounts = staking_options::cpi::accounts::Issue {
        authority: ctx.accounts.so_authority.to_account_info(),
        state: ctx.accounts.so_state.to_account_info(),
        option_mint: ctx.accounts.so_option_mint.to_account_info(),
        user_so_account: ctx.accounts.so_user_option_account.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };
    let cpi_program_issue = ctx.accounts.staking_options_program.to_account_info();

    staking_options::cpi::issue(
        CpiContext::new_with_signer(
            cpi_program_issue,
            so_issue_accounts,
            &[&[
                "gso".as_bytes(),
                &ctx.accounts.gso_state.key().to_bytes(),
                &[ctx.accounts.gso_state.so_authority_bump],
            ]],
        ),
        num_staking_options,
        ctx.accounts.gso_state.strike,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct GSOStake<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [GSO_STATE_SEED, &gso_state.period_num.to_be_bytes(), &gso_state.project_name.as_bytes()],
        bump = gso_state.gso_state_bump,
    )]
    pub gso_state: Box<Account<'info, GSOState>>,

    /// CHECK: Not dangerous. Just an AccountInfo for signing.
    #[account(mut,
        seeds = [SO_AUTHORITY_SEED, &gso_state.key().to_bytes()],
        bump = gso_state.so_authority_bump
    )]
    pub so_authority: AccountInfo<'info>,

    /// The so_option_mint is verified inside the SO CPI.
    #[account(mut)]
    pub so_option_mint: Account<'info, Mint>,
    #[account(mut)]
    pub so_user_option_account: Box<Account<'info, TokenAccount>>,
    /// Seeds are verified in the CPI. Verified that this is the correct
    /// so_state here.
    #[account(mut, constraint = so_state.key() == gso_state.staking_options_state)]
    pub so_state: Box<Account<'info, staking_options::State>>,

    pub staking_options_program: Program<'info, StakingOptionsProgram>,

    #[account(
        mut,
        seeds = [X_GSO_SEED, &gso_state.key().to_bytes()],
        bump = gso_state.x_base_mint_bump
    )]
    pub x_base_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub user_base_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub user_x_base_account: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        seeds = [BASE_VAULT_SEED, &gso_state.key().to_bytes()],
        bump = gso_state.base_vault_bump
    )]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}
