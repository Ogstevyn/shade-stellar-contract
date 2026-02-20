use crate::components::core;
use crate::events;
use crate::types::{DataKey, Role};
use soroban_sdk::{Address, Env};

pub fn grant_role(env: &Env, admin: &Address, user: &Address, role: Role) {
    core::assert_admin(env, admin);

    env.storage()
        .persistent()
        .set(&DataKey::Role(user.clone(), role.clone()), &true);

    events::publish_role_granted_event(env, user.clone(), role, env.ledger().timestamp());
}

pub fn revoke_role(env: &Env, admin: &Address, user: &Address, role: Role) {
    core::assert_admin(env, admin);

    env.storage()
        .persistent()
        .remove(&DataKey::Role(user.clone(), role.clone()));

    events::publish_role_revoked_event(env, user.clone(), role, env.ledger().timestamp());
}

pub fn has_role(env: &Env, user: &Address, role: Role) -> bool {
    let admin = core::get_admin(env);
    if *user == admin {
        return true;
    }

    env.storage()
        .persistent()
        .has(&DataKey::Role(user.clone(), role))
}

pub fn assert_has_role(env: &Env, user: &Address, role: Role) {
    user.require_auth();
    if !has_role(env, user, role) {
        use crate::errors::ContractError;
        use soroban_sdk::panic_with_error;
        panic_with_error!(env, ContractError::NotAuthorized);
    }
}
