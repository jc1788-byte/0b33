use soroban_sdk::{Env, Address};
use crate::types::{DataKey, Error};

/// Increases the globally tracked accounted amount for a token
pub fn add_total_accounted(env: &Env, token: &Address, amount: i128) -> Result<(), Error> {
    if amount < 0 {
        return Err(Error::InvalidAmount);
    }

    let key = DataKey::TotalAccounted(token.clone());
    let current: i128 = env.storage().instance().get(&key).unwrap_or(0i128);

    let new_value = current
        .checked_add(amount)
        .ok_or(Error::Overflow)?;

    env.storage().instance().set(&key, &new_value);
    Ok(())
}

/// Decreases the globally tracked accounted amount for a token
pub fn sub_total_accounted(env: &Env, token: &Address, amount: i128) -> Result<(), Error> {
    if amount < 0 {
        return Err(Error::InvalidAmount);
    }

    let key = DataKey::TotalAccounted(token.clone());
    let current: i128 = env.storage().instance().get(&key).unwrap_or(0i128);

    let new_value = current
        .checked_sub(amount)
        .ok_or(Error::Underflow)?;

    env.storage().instance().set(&key, &new_value);
    Ok(())
}

/// Reads total accounted value for a registered token.
pub fn get_total_accounted(env: &Env, token: &Address) -> Result<i128, Error> {
    if !crate::admin::is_token_accepted(env, token) {
        return Err(Error::InvalidToken);
    }

    let key = DataKey::TotalAccounted(token.clone());
    Ok(env.storage().instance().get(&key).unwrap_or(0i128))
}