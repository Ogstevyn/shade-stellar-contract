use crate::components::core;
use crate::events;
use soroban_sdk::{BytesN, Env};

pub fn upgrade(env: &Env, new_wasm_hash: &BytesN<32>) {
    let admin = core::get_admin(env);
    core::assert_admin(env, &admin);

    env.deployer()
        .update_current_contract_wasm(new_wasm_hash.clone());

    events::publish_contract_upgraded_event(env, new_wasm_hash.clone(), env.ledger().timestamp());
}
