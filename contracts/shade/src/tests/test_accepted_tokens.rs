#![cfg(test)]

use crate::components::admin as admin_component;
use crate::errors::ContractError;
use crate::shade::Shade;
use crate::shade::ShadeClient;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Address, Env, Map, Symbol, TryIntoVal, Val, Vec};

fn assert_latest_token_event(
    env: &Env,
    contract_id: &Address,
    expected_event: &str,
    expected_token: &Address,
    expected_timestamp: u64,
) {
    let events = env.events().all();
    assert!(!events.is_empty());

    let (event_contract_id, topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(event_contract_id, contract_id.clone());
    assert_eq!(topics.len(), 1);

    let event_name: Symbol = topics.get(0).unwrap().try_into_val(env).unwrap();
    assert_eq!(event_name, Symbol::new(env, expected_event));

    let data_map: Map<Symbol, Val> = data.try_into_val(env).unwrap();
    let token_val = data_map.get(Symbol::new(env, "token")).unwrap();
    let timestamp_val = data_map.get(Symbol::new(env, "timestamp")).unwrap();

    let token_in_event: Address = token_val.try_into_val(env).unwrap();
    let timestamp_in_event: u64 = timestamp_val.try_into_val(env).unwrap();

    assert_eq!(token_in_event, expected_token.clone());
    assert_eq!(timestamp_in_event, expected_timestamp);
}

#[test]
fn test_admin_adds_token_and_emits_event() {
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

    let expected_timestamp = env.ledger().timestamp();

    env.as_contract(&contract_id, || {
        admin_component::add_accepted_token(&env, &admin, &token);
        assert_latest_token_event(
            &env,
            &contract_id,
            "token_added_event",
            &token,
            expected_timestamp,
        );
    });

    assert!(client.is_accepted_token(&token));
}

#[test]
fn test_batch_add_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token1 = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token2 = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token3 = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();

    let mut tokens = Vec::new(&env);
    tokens.push_back(token1.clone());
    tokens.push_back(token2.clone());
    tokens.push_back(token3.clone());

    client.add_accepted_tokens(&admin, &tokens);

    assert!(client.is_accepted_token(&token1));
    assert!(client.is_accepted_token(&token2));
    assert!(client.is_accepted_token(&token3));
}

#[test]
fn test_admin_removes_token_and_emits_event() {
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

    env.as_contract(&contract_id, || {
        admin_component::add_accepted_token(&env, &admin, &token);
    });
    assert!(client.is_accepted_token(&token));

    let expected_timestamp = env.ledger().timestamp();

    env.as_contract(&contract_id, || {
        admin_component::remove_accepted_token(&env, &admin, &token);
        assert_latest_token_event(
            &env,
            &contract_id,
            "token_removed_event",
            &token,
            expected_timestamp,
        );
    });

    assert!(!client.is_accepted_token(&token));
}

#[test]
fn test_duplicate_add_is_handled_gracefully() {
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
    client.add_accepted_token(&admin, &token);

    assert!(client.is_accepted_token(&token));

    client.remove_accepted_token(&admin, &token);
    assert!(!client.is_accepted_token(&token));
}

#[test]
fn test_non_admin_cannot_add_or_remove_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    let expected_error =
        soroban_sdk::Error::from_contract_error(ContractError::NotAuthorized as u32);

    let add_result = client.try_add_accepted_token(&non_admin, &token);
    assert!(matches!(add_result, Err(Ok(err)) if err == expected_error));

    client.add_accepted_token(&admin, &token);
    let remove_result = client.try_remove_accepted_token(&non_admin, &token);
    assert!(matches!(remove_result, Err(Ok(err)) if err == expected_error));
    assert!(client.is_accepted_token(&token));
}

#[test]
#[should_panic]
fn test_invalid_token_address_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let invalid_token = Address::generate(&env);
    client.add_accepted_token(&admin, &invalid_token);
}
