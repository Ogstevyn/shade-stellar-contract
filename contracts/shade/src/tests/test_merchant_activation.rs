#![cfg(test)]

use crate::components::merchant as merchant_component;
use crate::errors::ContractError;
use crate::shade::Shade;
use crate::shade::ShadeClient;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Address, Env, Map, Symbol, TryIntoVal, Val};

fn assert_latest_merchant_status_event(
    env: &Env,
    contract_id: &Address,
    expected_merchant_id: u64,
    expected_active: bool,
    expected_timestamp: u64,
) {
    let events = env.events().all();
    assert!(!events.is_empty());

    let (event_contract_id, topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(event_contract_id, contract_id.clone());
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
fn test_successful_activation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let merchant_id = 1u64;
    assert!(client.is_merchant_active(&merchant_id));

    let merchant_data = client.get_merchant(&merchant_id);
    assert!(merchant_data.active);

    client.set_merchant_status(&admin, &merchant_id, &false);
    assert!(!client.is_merchant_active(&merchant_id));

    let expected_timestamp = env.ledger().timestamp();

    env.as_contract(&contract_id, || {
        merchant_component::set_merchant_status(&env, &admin, merchant_id, true);
        assert_latest_merchant_status_event(
            &env,
            &contract_id,
            merchant_id,
            true,
            expected_timestamp,
        );
    });

    assert!(client.is_merchant_active(&merchant_id));
    let updated_merchant = client.get_merchant(&merchant_id);
    assert!(updated_merchant.active);
}

#[test]
fn test_successful_deactivation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let merchant_id = 1u64;
    assert!(client.is_merchant_active(&merchant_id));

    let expected_timestamp = env.ledger().timestamp();

    env.as_contract(&contract_id, || {
        merchant_component::set_merchant_status(&env, &admin, merchant_id, false);
        assert_latest_merchant_status_event(
            &env,
            &contract_id,
            merchant_id,
            false,
            expected_timestamp,
        );
    });

    assert!(!client.is_merchant_active(&merchant_id));
    let merchant_data = client.get_merchant(&merchant_id);
    assert!(!merchant_data.active);
}

#[test]
fn test_unauthorized_status_change() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    client.initialize(&admin);

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let merchant_id = 1u64;

    let expected_error =
        soroban_sdk::Error::from_contract_error(ContractError::NotAuthorized as u32);

    let result = client.try_set_merchant_status(&non_admin, &merchant_id, &false);
    assert!(matches!(result, Err(Ok(err)) if err == expected_error));

    assert!(client.is_merchant_active(&merchant_id));
}

#[test]
#[should_panic]
fn test_invalid_merchant_id_status_change() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let invalid_merchant_id = 999u64;
    client.set_merchant_status(&admin, &invalid_merchant_id, &true);
}

#[test]
#[should_panic]
fn test_is_merchant_active_invalid_id() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let invalid_merchant_id = 999u64;
    client.is_merchant_active(&invalid_merchant_id);
}

#[test]
#[should_panic]
fn test_set_merchant_status_zero_id() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let zero_merchant_id = 0u64;
    client.set_merchant_status(&admin, &zero_merchant_id, &true);
}

#[test]
#[should_panic]
fn test_is_merchant_active_zero_id() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let zero_merchant_id = 0u64;
    client.is_merchant_active(&zero_merchant_id);
}
