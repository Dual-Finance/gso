use anchor_spl::token::{Mint, Token, TokenAccount};
use staking_options::program::StakingOptions as StakingOptionsProgram;

pub use crate::common::*;
pub use crate::errors::ErrorCode;
pub use crate::*;

pub fn config_v2(
    ctx: Context<GSOConfigV2>,
    // GSO Params
    period_num: u64,
    lockup_ratio_tokens_per_million: u64,
    lockup_period_end: u64,
    // SO Config params
    option_expiration: u64,
    subscription_period_end: u64,
    lot_size: u64,
    num_tokens: u64,
    project_name: String,
    // SO Init Strike params
    strike: u64,
    // SO authority params
    so_authority_bump: u8,
) -> Result<()> {
    msg!("GSO Config");

    invariant!(
        lockup_period_end >= subscription_period_end,
        InvalidLockupEnd
    );

    msg!("SO Config");
    let so_config_accounts = staking_options::cpi::accounts::Config {
        authority: ctx.accounts.authority.to_account_info(),
        so_authority: ctx.accounts.so_authority.to_account_info(),
        state: ctx.accounts.so_state.to_account_info(),
        base_vault: ctx.accounts.so_base_vault.to_account_info(),
        base_account: ctx.accounts.so_base_account.to_account_info(),
        quote_account: ctx.accounts.so_quote_account.to_account_info(),
        base_mint: ctx.accounts.so_base_mint.to_account_info(),
        quote_mint: ctx.accounts.so_quote_mint.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_program_config = ctx.accounts.staking_options_program.to_account_info();

    staking_options::cpi::config(
        CpiContext::new_with_signer(
            cpi_program_config,
            so_config_accounts,
            &[&[
                SO_AUTHORITY_SEED,
                &ctx.accounts.gso_state.key().to_bytes(),
                &[so_authority_bump],
            ]],
        ),
        option_expiration,
        subscription_period_end,
        num_tokens,
        lot_size,
        format!("{}{}", "GSO", project_name),
    )?;

    msg!("SO Init Strike");
    let so_init_strike_accounts = staking_options::cpi::accounts::InitStrikeWithPayer {
        authority: ctx.accounts.so_authority.to_account_info(),
        payer: ctx.accounts.authority.to_account_info(),
        state: ctx.accounts.so_state.to_account_info(),
        option_mint: ctx.accounts.so_option_mint.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_program_init_strike = ctx.accounts.staking_options_program.to_account_info();

    staking_options::cpi::init_strike_with_payer(
        CpiContext::new_with_signer(
            cpi_program_init_strike,
            so_init_strike_accounts,
            &[&[
                SO_AUTHORITY_SEED,
                &ctx.accounts.gso_state.key().to_bytes(),
                &[so_authority_bump],
            ]],
        ),
        strike,
    )?;

    msg!("GSO config params");
    // Store the bump for the GSO State and other values for the GSO wrapper.
    ctx.accounts.gso_state.period_num = period_num;
    ctx.accounts.gso_state.strike = strike;
    ctx.accounts.gso_state.lockup_ratio_tokens_per_million = lockup_ratio_tokens_per_million;
    ctx.accounts.gso_state.gso_state_bump = *ctx.bumps.get("gso_state").unwrap();
    ctx.accounts.gso_state.x_base_mint_bump = *ctx.bumps.get("x_base_mint").unwrap();
    ctx.accounts.gso_state.so_authority_bump = so_authority_bump;
    ctx.accounts.gso_state.base_vault_bump = *ctx.bumps.get("base_vault").unwrap();
    ctx.accounts.gso_state.staking_options_state = ctx.accounts.so_state.key();
    ctx.accounts.gso_state.project_name = project_name;
    ctx.accounts.gso_state.subscription_period_end = subscription_period_end;
    ctx.accounts.gso_state.authority = ctx.accounts.authority.key();
    ctx.accounts.gso_state.base_mint = ctx.accounts.so_base_mint.key();
    ctx.accounts.gso_state.lockup_period_end = lockup_period_end;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    // GSO Params
    period_num: u64,
    lockup_ratio_tokens_per_million: u64,
    lockup_period_end: u64,
    // SO Config params
    option_expiration: u64,
    subscription_period_end: u64,
    lot_size: u64,
    num_tokens: u64,
    project_name: String,
    // SO Init Strike params
    strike: u64,
    so_authority_bump: u8,
)]
pub struct GSOConfigV2<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        seeds = [GSO_STATE_SEED, &period_num.to_be_bytes(), project_name.as_bytes()],
        bump,
        space = 1_000 // Plenty of padding
    )]
    pub gso_state: Box<Account<'info, GSOState>>,

    /// SO Config
    /// =========
    /// CHECK: Not dangerous. This is just a PDA, not a funded account.
    #[account(mut,
        seeds = [SO_AUTHORITY_SEED, &gso_state.key().to_bytes()],
        bump = so_authority_bump,
    )]
    pub so_authority: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: Not dangerous. Checked in CPI where it is initialized.
    pub so_state: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Not dangerous. Checked in CPI where it is initialized.
    pub so_base_vault: UncheckedAccount<'info>,
    #[account(mut)]
    pub so_base_account: Box<Account<'info, TokenAccount>>,
    pub so_quote_account: Box<Account<'info, TokenAccount>>,

    pub so_base_mint: Box<Account<'info, Mint>>,
    pub so_quote_mint: Box<Account<'info, Mint>>,

    /// SO Init Strike
    /// =========
    #[account(mut)]
    /// CHECK: Not dangerous. Checked in CPI where it is initiailized.
    pub so_option_mint: UncheckedAccount<'info>,

    pub staking_options_program: Program<'info, StakingOptionsProgram>,

    #[account(
        init,
        payer = authority,
        seeds = [X_GSO_SEED, &gso_state.key().to_bytes()],
        bump,
        mint::decimals = so_base_mint.decimals,
        mint::authority = x_base_mint)]
    pub x_base_mint: Box<Account<'info, Mint>>,

    // This is the difference with v1, the lockup mint can be different from the
    // SO base mint.
    pub lockup_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = authority,
        seeds = [BASE_VAULT_SEED, &gso_state.key().to_bytes()],
        token::mint = lockup_mint,
        token::authority = base_vault,
        bump
    )]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
