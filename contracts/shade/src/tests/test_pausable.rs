#![cfg(test)]

use crate::components::pausable as pausable_component;
use crate::errors::ContractError;
use crate::shade::{Shade, ShadeClient};
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Address, Env, Map, Symbol, TryIntoVal, Val};

fn setup_test() -> (Env, ShadeClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, client, contract_id, admin)
}

fn assert_latest_pause_event(
    env: &Env,
    contract_id: &Address,
    expected_event: &str,
    expected_admin: &Address,
    expected_timestamp: u64,
) {
    let events = env.events().all();
    assert!(!events.is_empty());

    let (event_contract_id, topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(&event_contract_id, contract_id);
    assert_eq!(topics.len(), 1);

    let event_name: Symbol = topics.get(0).unwrap().try_into_val(env).unwrap();
    assert_eq!(event_name, Symbol::new(env, expected_event));

    let data_map: Map<Symbol, Val> = data.try_into_val(env).unwrap();
    let admin_val = data_map.get(Symbol::new(env, "admin")).unwrap();
    let timestamp_val = data_map.get(Symbol::new(env, "timestamp")).unwrap();

    let admin_in_event: Address = admin_val.try_into_val(env).unwrap();
    let timestamp_in_event: u64 = timestamp_val.try_into_val(env).unwrap();

    assert_eq!(admin_in_event, expected_admin.clone());
    assert_eq!(timestamp_in_event, expected_timestamp);
}

#[test]
fn test_admin_can_pause_and_unpause() {
    let (_env, client, _contract_id, admin) = setup_test();

    assert!(!client.is_paused());

    client.pause(&admin);
    assert!(client.is_paused());

    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
fn test_non_admin_cannot_pause_or_unpause() {
    let (_env, client, _contract_id, admin) = setup_test();
    let non_admin = Address::generate(&_env);

    let expected_error =
        soroban_sdk::Error::from_contract_error(ContractError::NotAuthorized as u32);

    let pause_result = client.try_pause(&non_admin);
    assert!(matches!(pause_result, Err(Ok(err)) if err == expected_error));

    client.pause(&admin);

    let unpause_result = client.try_unpause(&non_admin);
    assert!(matches!(unpause_result, Err(Ok(err)) if err == expected_error));

    assert!(client.is_paused());
}

#[test]
fn test_is_paused_state_transitions_are_accurate() {
    let (_env, client, _contract_id, admin) = setup_test();

    assert!(!client.is_paused());

    client.pause(&admin);
    assert!(client.is_paused());

    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
fn test_pause_and_unpause_emit_expected_events() {
    let (env, client, contract_id, admin) = setup_test();

    let paused_timestamp = env.ledger().timestamp();
    client.pause(&admin);
    assert_latest_pause_event(
        &env,
        &contract_id,
        "contract_paused_event",
        &admin,
        paused_timestamp,
    );

    let unpaused_timestamp = env.ledger().timestamp();
    client.unpause(&admin);
    assert_latest_pause_event(
        &env,
        &contract_id,
        "contract_unpaused_event",
        &admin,
        unpaused_timestamp,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_assert_paused_panics_when_not_paused() {
    let (env, _client, contract_id, _admin) = setup_test();

    env.as_contract(&contract_id, || {
        pausable_component::assert_paused(&env);
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_assert_not_paused_panics_when_paused() {
    let (env, client, contract_id, admin) = setup_test();
    client.pause(&admin);

    env.as_contract(&contract_id, || {
        pausable_component::assert_not_paused(&env);
    });
}

#[test]
fn test_double_pause_fails_with_contract_paused_error() {
    let (_env, client, _contract_id, admin) = setup_test();

    client.pause(&admin);

    let expected_error =
        soroban_sdk::Error::from_contract_error(ContractError::ContractPaused as u32);
    let pause_again_result = client.try_pause(&admin);

    assert!(matches!(pause_again_result, Err(Ok(err)) if err == expected_error));
    assert!(client.is_paused());
}

#[test]
fn test_double_unpause_fails_with_contract_not_paused_error() {
    let (_env, client, _contract_id, admin) = setup_test();

    let expected_error =
        soroban_sdk::Error::from_contract_error(ContractError::ContractNotPaused as u32);
    let unpause_result = client.try_unpause(&admin);

    assert!(matches!(unpause_result, Err(Ok(err)) if err == expected_error));
    assert!(!client.is_paused());
}
