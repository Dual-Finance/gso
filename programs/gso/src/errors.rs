use anchor_lang::prelude::*;

// Most errors are just propagated through the SO calls and token program. So
// really the only errors are the ones in unstaking.

#[error_code]
pub enum ErrorCode {
    #[msg("Attempted to perform an action for a vault not yet expired")]
    NotYetExpired,
    #[msg("Lockup end cannot be before subscription period end")]
    InvalidLockupEnd,
}
