use anchor_lang::prelude::*;

pub const GSO_STATE_SEED: &[u8] = b"GSO-state";
pub const X_GSO_SEED: &[u8] = b"xGSO";
pub const BASE_VAULT_SEED: &[u8] = b"base-vault";
pub const SO_AUTHORITY_SEED: &[u8] = b"gso";

#[account]
pub struct GSOState {
    // Number of the period. Starts at 0. Not strictly enforced.
    pub period_num: u64,

    // Matches the SO subscriptoin period end.
    pub subscription_period_end: u64,

    // Lockup ratio for how many options the user gets for every staked token.
    pub lockup_ratio_tokens_per_million: u64,

    // Bumps, mainly used for account validation.
    pub gso_state_bump: u8,
    pub so_authority_bump: u8,
    pub x_base_mint_bump: u8,
    pub base_vault_bump: u8,

    // Strike in atoms for the quote.
    pub strike: u64,

    // Name in this state gets prepended with "GSO" and is then used at the SO
    // level.
    pub project_name: String,

    // Just an address, used for account validation.
    pub staking_options_state: Pubkey,

    // Saved so we know who can withdraw.
    pub authority: Pubkey,

    // Base mint stored so it does not need to be on the.
    pub base_mint: Pubkey,

    // Time in seconds for when users can unstake.
    pub lockup_period_end: u64,
    // Padding
}
