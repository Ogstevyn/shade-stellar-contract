#![cfg(test)]
use crate::shade::{Shade, ShadeClient};
use crate::types::DataKey;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Address, BytesN, Env, Map, Symbol, TryIntoVal, Val, Vec};

const V2_WASM: &[u8] = include_bytes!("fixtures/upgrade_v2_contract.wasm");

fn assert_latest_upgrade_event(
    env: &Env,
    contract_id: &Address,
    expected_hash: &BytesN<32>,
    expected_timestamp: u64,
) {
    let events = env.events().all();
    assert!(!events.is_empty());

    let (event_contract_id, topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(event_contract_id, contract_id.clone());
    assert_eq!(topics.len(), 1);

    let event_name: Symbol = topics.get(0).unwrap().try_into_val(env).unwrap();
    assert_eq!(event_name, Symbol::new(env, "contract_upgraded_event"));

    let data_map: Map<Symbol, Val> = data.try_into_val(env).unwrap();
    let hash_val = data_map.get(Symbol::new(env, "new_wasm_hash")).unwrap();
    let timestamp_val = data_map.get(Symbol::new(env, "timestamp")).unwrap();

    let hash_in_event: BytesN<32> = hash_val.try_into_val(env).unwrap();
    let timestamp_in_event: u64 = timestamp_val.try_into_val(env).unwrap();

    assert_eq!(hash_in_event, expected_hash.clone());
    assert_eq!(timestamp_in_event, expected_timestamp);
}

#[test]
fn test_admin_can_upgrade_successfully() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let v2_hash = env.deployer().upload_contract_wasm(V2_WASM);
    client.upgrade(&v2_hash);
}

#[test]
fn test_state_persists_after_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    client.add_accepted_token(&admin, &token);

    let v2_hash = env.deployer().upload_contract_wasm(V2_WASM);
    client.upgrade(&v2_hash);

    let stored_admin: Address = env.as_contract(&contract_id, || {
        env.storage().persistent().get(&DataKey::Admin).unwrap()
    });
    assert_eq!(stored_admin, admin);

    let accepted_tokens: Vec<Address> = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get(&DataKey::AcceptedTokens)
            .unwrap()
    });
    let mut token_found = false;
    for accepted_token in accepted_tokens.iter() {
        if accepted_token == token {
            token_found = true;
            break;
        }
    }
    assert!(token_found);
}

#[test]
fn test_upgrade_emits_contract_upgraded_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let v2_hash = env.deployer().upload_contract_wasm(V2_WASM);
    let expected_timestamp = env.ledger().timestamp();

    client.upgrade(&v2_hash);
    assert_latest_upgrade_event(&env, &contract_id, &v2_hash, expected_timestamp);
}
