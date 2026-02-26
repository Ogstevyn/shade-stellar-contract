#![cfg(test)]

use crate::components::admin as admin_component;
use crate::errors::ContractError;
use crate::shade::Shade;
use crate::shade::ShadeClient;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Address, Env, Map, Symbol, TryIntoVal, Val};

fn setup_with_accepted_token(env: &Env) -> (Address, ShadeClient<'_>, Address) {
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.initialize(&admin);

    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.add_accepted_token(&admin, &token);

    (admin, client, token)
}

fn assert_fee_set_event(
    env: &Env,
    contract_id: &Address,
    expected_token: &Address,
    expected_fee: i128,
    expected_timestamp: u64,
) {
    let events = env.events().all();
    assert!(!events.is_empty());

    let (event_contract_id, topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(event_contract_id, contract_id.clone());
    assert_eq!(topics.len(), 1);

    let event_name: Symbol = topics.get(0).unwrap().try_into_val(env).unwrap();
    assert_eq!(event_name, Symbol::new(env, "fee_set_event"));

    let data_map: Map<Symbol, Val> = data.try_into_val(env).unwrap();
    let token_val = data_map.get(Symbol::new(env, "token")).unwrap();
    let fee_val = data_map.get(Symbol::new(env, "fee")).unwrap();
    let timestamp_val = data_map.get(Symbol::new(env, "timestamp")).unwrap();

    let token_in_event: Address = token_val.try_into_val(env).unwrap();
    let fee_in_event: i128 = fee_val.try_into_val(env).unwrap();
    let timestamp_in_event: u64 = timestamp_val.try_into_val(env).unwrap();

    assert_eq!(token_in_event, expected_token.clone());
    assert_eq!(fee_in_event, expected_fee);
    assert_eq!(timestamp_in_event, expected_timestamp);
}

#[test]
fn test_set_fee_success() {
    let env = Env::default();
    let (admin, client, token) = setup_with_accepted_token(&env);
    let contract_id = client.address.clone();
    let fee: i128 = 500;

    let expected_timestamp = env.ledger().timestamp();

    env.as_contract(&contract_id, || {
        admin_component::set_fee(&env, &admin, &token, fee);
        assert_fee_set_event(&env, &contract_id, &token, fee, expected_timestamp);
    });

    assert_eq!(client.get_fee(&token), fee);
}

#[test]
fn test_set_fee_unaccepted_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let unaccepted_token = Address::generate(&env);

    let expected_error =
        soroban_sdk::Error::from_contract_error(ContractError::TokenNotAccepted as u32);

    let result = client.try_set_fee(&admin, &unaccepted_token, &100);
    assert!(matches!(result, Err(Ok(err)) if err == expected_error));
}

#[test]
fn test_set_fee_unauthorized() {
    let env = Env::default();
    let (_admin, client, token) = setup_with_accepted_token(&env);

    let non_admin = Address::generate(&env);

    let expected_error =
        soroban_sdk::Error::from_contract_error(ContractError::NotAuthorized as u32);

    let result = client.try_set_fee(&non_admin, &token, &100);
    assert!(matches!(result, Err(Ok(err)) if err == expected_error));
}

#[test]
fn test_update_fee() {
    let env = Env::default();
    let (admin, client, token) = setup_with_accepted_token(&env);

    client.set_fee(&admin, &token, &200);
    assert_eq!(client.get_fee(&token), 200);

    client.set_fee(&admin, &token, &750);
    assert_eq!(client.get_fee(&token), 750);
}

#[test]
fn test_get_fee_default_zero() {
    let env = Env::default();
    let (_admin, client, token) = setup_with_accepted_token(&env);

    assert_eq!(client.get_fee(&token), 0);
}
