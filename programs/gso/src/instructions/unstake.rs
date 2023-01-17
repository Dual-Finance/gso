use anchor_spl::token::{Mint, Token, TokenAccount};

pub use crate::common::*;
pub use crate::errors::ErrorCode;
pub use crate::*;

pub fn unstake(ctx: Context<GSOUnstake>, amount: u64) -> Result<()> {
    msg!("GSO Unstake");
    let now_ts: u64 = Clock::get().unwrap().unix_timestamp as u64;
    let expiration: u64 = ctx.accounts.gso_state.lockup_period_end;
    msg!("Now {} Expiration {}", now_ts, expiration);
    invariant!(expiration < now_ts, NotYetExpired);

    msg!("Burn xTokens");
    anchor_spl::token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.x_base_mint.to_account_info(),
                from: ctx.accounts.user_x_base_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        amount,
    )?;

    msg!("Return tokens");
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.base_vault.to_account_info(),
                to: ctx.accounts.user_base_account.to_account_info(),
                authority: ctx.accounts.base_vault.to_account_info().clone(),
            },
            &[&[
                BASE_VAULT_SEED,
                &ctx.accounts.gso_state.key().to_bytes(),
                &[ctx.accounts.gso_state.base_vault_bump],
            ]],
        ),
        amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct GSOUnstake<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [GSO_STATE_SEED, &gso_state.period_num.to_be_bytes(), &gso_state.project_name.as_bytes()],
        bump = gso_state.gso_state_bump,
    )]
    pub gso_state: Box<Account<'info, GSOState>>,

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
