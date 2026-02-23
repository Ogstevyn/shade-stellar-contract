#![cfg(test)]

use crate::components::merchant as merchant_component;
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

fn assert_latest_merchant_status_event(
    env: &Env,
    contract_id: &Address,
    expected_merchant_id: u64,
    expected_active: bool,
    expected_timestamp: u64,
) {
    let events = env.events().all();
    assert!(events.len() > 0);

    let (event_contract_id, topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(&event_contract_id, contract_id);
    assert_eq!(topics.len(), 1);

    let event_name: Symbol = topics.get(0).unwrap().try_into_val(env).unwrap();
    assert_eq!(
        event_name,
        Symbol::new(env, "merchant_status_changed_event")
    );

    let data_map: Map<Symbol, Val> = data.try_into_val(env).unwrap();
    let merchant_id_val = data_map.get(Symbol::new(env, "merchant_id")).unwrap();
    let active_val = data_map.get(Symbol::new(env, "active")).unwrap();
    let timestamp_val = data_map.get(Symbol::new(env, "timestamp")).unwrap();

    let merchant_id_in_event: u64 = merchant_id_val.try_into_val(env).unwrap();
    let active_in_event: bool = active_val.try_into_val(env).unwrap();
    let timestamp_in_event: u64 = timestamp_val.try_into_val(env).unwrap();

    assert_eq!(merchant_id_in_event, expected_merchant_id);
    assert_eq!(active_in_event, expected_active);
    assert_eq!(timestamp_in_event, expected_timestamp);
}

#[test]
fn test_set_merchant_status_admin_can_deactivate() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    // Merchant should be active by default
    assert_eq!(client.is_merchant_active(&1), true);

    let expected_timestamp = env.ledger().timestamp();

    env.as_contract(&contract_id, || {
        merchant_component::set_merchant_status(&env, &admin, 1, false);
        assert_latest_merchant_status_event(&env, &contract_id, 1, false, expected_timestamp);
    });

    assert_eq!(client.is_merchant_active(&1), false);
}

#[test]
fn test_set_merchant_status_admin_can_activate() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    // Deactivate first
    env.as_contract(&contract_id, || {
        merchant_component::set_merchant_status(&env, &admin, 1, false);
    });
    assert_eq!(client.is_merchant_active(&1), false);

    // Now activate
    let expected_timestamp = env.ledger().timestamp();

    env.as_contract(&contract_id, || {
        merchant_component::set_merchant_status(&env, &admin, 1, true);
        assert_latest_merchant_status_event(&env, &contract_id, 1, true, expected_timestamp);
    });

    assert_eq!(client.is_merchant_active(&1), true);
}

#[should_panic(expected = "HostError: Error(Contract, #1)")]
#[test]
fn test_set_merchant_status_non_admin_not_authorized() {
    let (env, client, _contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let non_admin = Address::generate(&env);
    client.set_merchant_status(&non_admin, &1, &false);
}

#[should_panic(expected = "HostError: Error(Contract, #6)")]
#[test]
fn test_set_merchant_status_invalid_merchant_id() {
    let (_env, client, _contract_id, admin) = setup_test();

    // Try to set status for non-existent merchant ID
    client.set_merchant_status(&admin, &999, &false);
}

#[should_panic(expected = "HostError: Error(Contract, #6)")]
#[test]
fn test_set_merchant_status_merchant_id_zero() {
    let (_env, client, _contract_id, admin) = setup_test();

    // Try to set status for merchant ID 0
    client.set_merchant_status(&admin, &0, &false);
}

#[should_panic(expected = "HostError: Error(Contract, #6)")]
#[test]
fn test_is_merchant_active_merchant_not_found() {
    let (_env, client, _contract_id, _admin) = setup_test();

    // Try to check status for non-existent merchant
    client.is_merchant_active(&999);
}

#[should_panic(expected = "HostError: Error(Contract, #6)")]
#[test]
fn test_is_merchant_active_merchant_id_zero() {
    let (_env, client, _contract_id, _admin) = setup_test();

    // Try to check status for merchant ID 0
    client.is_merchant_active(&0);
}

#[test]
fn test_merchant_active_by_default() {
    let (env, client, _contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    // Newly registered merchant should be active
    assert_eq!(client.is_merchant_active(&1), true);
}

#[test]
fn test_multiple_merchants_independent_status() {
    let (env, client, _contract_id, admin) = setup_test();

    let merchant1 = Address::generate(&env);
    let merchant2 = Address::generate(&env);

    client.register_merchant(&merchant1);
    client.register_merchant(&merchant2);

    // Both should be active
    assert_eq!(client.is_merchant_active(&1), true);
    assert_eq!(client.is_merchant_active(&2), true);

    // Deactivate merchant 1
    client.set_merchant_status(&admin, &1, &false);

    // Check they have independent status
    assert_eq!(client.is_merchant_active(&1), false);
    assert_eq!(client.is_merchant_active(&2), true);

    // Reactivate merchant 1, merchant 2 should remain active
    client.set_merchant_status(&admin, &1, &true);
    assert_eq!(client.is_merchant_active(&1), true);
    assert_eq!(client.is_merchant_active(&2), true);
}

#[test]
fn test_event_emission_on_status_change() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let expected_timestamp = env.ledger().timestamp();

    env.as_contract(&contract_id, || {
        merchant_component::set_merchant_status(&env, &admin, 1, false);
        assert_latest_merchant_status_event(&env, &contract_id, 1, false, expected_timestamp);
    });
}
