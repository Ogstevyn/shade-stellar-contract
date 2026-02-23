#![cfg(test)]

use crate::components::merchant as merchant_component;
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

fn assert_latest_merchant_verified_event(
    env: &Env,
    contract_id: &Address,
    expected_merchant_id: u64,
    expected_status: bool,
    expected_timestamp: u64,
) {
    let events = env.events().all();
    assert!(events.len() > 0);

    let (event_contract_id, topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(&event_contract_id, contract_id);
    assert_eq!(topics.len(), 1);

    let event_name: Symbol = topics.get(0).unwrap().try_into_val(env).unwrap();
    assert_eq!(event_name, Symbol::new(env, "merchant_verified_event"));

    let data_map: Map<Symbol, Val> = data.try_into_val(env).unwrap();
    let merchant_id_val = data_map.get(Symbol::new(env, "merchant_id")).unwrap();
    let status_val = data_map.get(Symbol::new(env, "status")).unwrap();
    let timestamp_val = data_map.get(Symbol::new(env, "timestamp")).unwrap();

    let merchant_id_in_event: u64 = merchant_id_val.try_into_val(env).unwrap();
    let status_in_event: bool = status_val.try_into_val(env).unwrap();
    let timestamp_in_event: u64 = timestamp_val.try_into_val(env).unwrap();

    assert_eq!(merchant_id_in_event, expected_merchant_id);
    assert_eq!(status_in_event, expected_status);
    assert_eq!(timestamp_in_event, expected_timestamp);
}

#[test]
fn test_successful_merchant_verification() {
    let (env, client, contract_id, admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let merchant_id = 1u64;
    let expected_timestamp = env.ledger().timestamp();

    env.as_contract(&contract_id, || {
        merchant_component::verify_merchant(&env, &admin, merchant_id, true);
        assert_latest_merchant_verified_event(
            &env,
            &contract_id,
            merchant_id,
            true,
            expected_timestamp,
        );
    });

    let merchant_data = client.get_merchant(&merchant_id);
    assert!(merchant_data.verified);
    assert!(client.is_merchant_verified(&merchant_id));
}

#[test]
fn test_non_admin_cannot_verify_merchant() {
    let (env, client, _contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let non_admin = Address::generate(&env);
    let expected_error =
        soroban_sdk::Error::from_contract_error(ContractError::NotAuthorized as u32);

    let result = client.try_verify_merchant(&non_admin, &1u64, &true);
    assert!(matches!(result, Err(Ok(err)) if err == expected_error));

    let merchant_data = client.get_merchant(&1u64);
    assert!(!merchant_data.verified);
}

#[should_panic(expected = "HostError: Error(Contract, #6)")]
#[test]
fn test_verify_non_existent_merchant_id_panics() {
    let (_env, client, _contract_id, admin) = setup_test();

    client.verify_merchant(&admin, &999u64, &true);
}

#[should_panic(expected = "HostError: Error(Contract, #6)")]
#[test]
fn test_is_merchant_verified_non_existent_id_panics() {
    let (_env, client, _contract_id, _admin) = setup_test();

    client.is_merchant_verified(&999u64);
}
