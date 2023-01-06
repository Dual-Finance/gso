use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use staking_options::program::StakingOptions as StakingOptionsProgram;

pub use crate::common::*;

pub fn name_tokens(ctx: Context<GSONameTokens>) -> Result<()> {
    msg!("SO Name Tokens");
    let so_name_token_accounts = staking_options::cpi::accounts::NameToken {
        authority: ctx.accounts.so_authority.to_account_info(),
        payer: ctx.accounts.authority.to_account_info(),
        state: ctx.accounts.so_state.to_account_info(),
        option_mint: ctx.accounts.so_option_mint.to_account_info(),
        option_mint_metadata_account: ctx.accounts.option_metadata.to_account_info(),
        token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_program_name = ctx.accounts.staking_options_program.to_account_info();

    staking_options::cpi::name_token(
        CpiContext::new_with_signer(
            cpi_program_name,
            so_name_token_accounts,
            &[&[
                SO_AUTHORITY_SEED,
                &ctx.accounts.gso_state.key().to_bytes(),
                &[ctx.accounts.gso_state.so_authority_bump],
            ]],
        ),
        ctx.accounts.so_state.strikes[0],
    )?;

    let token_name: String = format!("DUAL-GSO-{:.15}", ctx.accounts.gso_state.project_name);
    let symbol: String = format!("DUAL-GSO");
    msg!("GSO Name Token for collateral");
    let ix = mpl_token_metadata::instruction::create_metadata_accounts_v3(
        mpl_token_metadata::ID,
        *ctx.accounts.x_base_metadata.key,
        ctx.accounts.x_base_mint.key(),
        ctx.accounts.x_base_mint.key(),
        ctx.accounts.authority.key(),
        ctx.accounts.x_base_mint.key(),
        token_name,
        symbol,
        "https://www.dual.finance/images/token-logos/gso-collateral".to_string(),
        None,
        0,
        true,
        true,
        None,
        None,
        None,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.x_base_metadata.to_account_info(),
            ctx.accounts.x_base_mint.to_account_info(),
            ctx.accounts.x_base_mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.x_base_mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        &[&[
            X_GSO_SEED,
            &ctx.accounts.gso_state.key().to_bytes(),
            &[ctx.accounts.gso_state.x_base_mint_bump],
        ]],
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct GSONameTokens<'info> {
    #[account(mut, constraint = authority.key() == gso_state.authority.key())]
    pub authority: Signer<'info>,

    #[account(constraint = so_state.key() == gso_state.staking_options_state)]
    pub so_state: Box<Account<'info, staking_options::State>>,

    #[account(
        seeds = [GSO_STATE_SEED, &gso_state.period_num.to_be_bytes(), &gso_state.project_name.as_bytes()],
        bump = gso_state.gso_state_bump,
    )]
    pub gso_state: Box<Account<'info, GSOState>>,

    #[account(
        seeds = [X_GSO_SEED, &gso_state.key().to_bytes()],
        bump = gso_state.x_base_mint_bump,
    )]
    pub x_base_mint: Box<Account<'info, Mint>>,

    /// CHECK: This is not dangerous. Checked by metaplex
    #[account(mut)]
    pub x_base_metadata: AccountInfo<'info>,

    /// CHECK: This is the metaplex program
    pub token_metadata_program: AccountInfo<'info>,

    /// CHECK: Not dangerous. This is just a PDA, not a funded account.
    #[account(mut,
        seeds = [SO_AUTHORITY_SEED, &gso_state.key().to_bytes()],
        bump = gso_state.so_authority_bump,
    )]
    pub so_authority: AccountInfo<'info>,

    /// Checked in the CPI
    pub so_option_mint: Box<Account<'info, Mint>>,

    /// CHECK: This is not dangerous. Checked by metaplex
    #[account(mut)]
    pub option_metadata: AccountInfo<'info>,

    pub staking_options_program: Program<'info, StakingOptionsProgram>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
