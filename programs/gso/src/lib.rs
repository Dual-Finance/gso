use anchor_lang::prelude::*;
use vipers::prelude::*;

mod common;
mod errors;
mod instructions;

pub use crate::common::*;
pub use crate::errors::ErrorCode;
pub use crate::instructions::*;

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "Dual GSO",
    project_url: "http://dual.finance",
    contacts: "email:dual-labs@dual.finance",
    policy: "https://github.com/Dual-Finance/gso/blob/master/SECURITY.md",

    preferred_languages: "en",
    source_code: "https://github.com/Dual-Finance/gso",
    auditors: "None"
}

declare_id!("DuALd6fooWzVDkaTsQzDAxPGYCnLrnWamdNNTNxicdX8");

#[program]
pub mod gso {
    use super::*;

    // The function of GSO is to provide a wrapper around StakingOptions with
    // the additional state and functionality for locking tokens. In the future,
    // capability may be added so that GSO staked tokens can be used by the
    // project in some way, like insurance or loans with slashing.

    // Note that GSO is a legacy name, it is a less general version of staking
    // options. GSO is a specific implementation of a use case of the SO
    // primitive.
        // Config. Minimal management in the GSO wrapper, most of the config work is
    // done in staking options itself.
    pub fn config(
        ctx: Context<GSOConfig>,
        period_num: u64,
        lockup_ratio_tokens_per_million: u64,
        lockup_period_end: u64,
        option_expiration: u64,
        subscription_period_end: u64,
        lot_size: u64,
        num_tokens: u64,
        project_name: String,
        strike_price: u64,
        so_authority_bump: u8,
    ) -> Result<()> {
        config::config(
            ctx,
            period_num,
            lockup_ratio_tokens_per_million,
            lockup_period_end,
            option_expiration,
            subscription_period_end,
            lot_size,
            num_tokens,
            project_name,
            strike_price,
            so_authority_bump,
        )
    }
    
    // ConfigV2. Same as config except that the SO base mint does not need to be
    // the same as the lockup mint.
    pub fn config_v2(
        ctx: Context<GSOConfigV2>,
        period_num: u64,
        lockup_ratio_tokens_per_million: u64,
        lockup_period_end: u64,
        option_expiration: u64,
        subscription_period_end: u64,
        lot_size: u64,
        num_tokens: u64,
        project_name: String,
        strike_price: u64,
        so_authority_bump: u8,
    ) -> Result<()> {
        config_v2::config_v2(
            ctx,
            period_num,
            lockup_ratio_tokens_per_million,
            lockup_period_end,
            option_expiration,
            subscription_period_end,
            lot_size,
            num_tokens,
            project_name,
            strike_price,
            so_authority_bump,
        )
    }

    // Stake. This is a liquid staking, so the user is able to split up and sell
    // their claim to their tokens back at the end of the staking period. The
    // receipt tokens of the base token deposit are xBaseTokens.
    pub fn stake(ctx: Context<GSOStake>, amount: u64) -> Result<()> {
        stake::stake(ctx, amount)
    }

    // Unstake. The holder of the receipt tokens from staking is able to redeem
    // for their tokens back. There is no verification that it is the same as
    // the depositor since it is a liquid staking.
    pub fn unstake(ctx: Context<GSOUnstake>, amount: u64) -> Result<()> {
        unstake::unstake(ctx, amount)
    }

    pub fn withdraw(ctx: Context<GSOWithdraw>) -> Result<()> {
        withdraw::withdraw(ctx)
    }

    pub fn name_tokens(ctx: Context<GSONameTokens>) -> Result<()> {
        name_tokens::name_tokens(ctx)
    }
}
