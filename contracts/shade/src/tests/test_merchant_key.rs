#![cfg(test)]

use crate::shade::{Shade, ShadeClient};
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Address, BytesN, Env, Map, Symbol, TryIntoVal, Val};

fn setup_test() -> (Env, ShadeClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, contract_id, admin)
}

#[test]
fn test_set_merchant_key_success() {
    let (env, client, contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let key = BytesN::from_array(&env, &[0u8; 32]);
    client.set_merchant_key(&merchant, &key);
    let events = env.events().all();

    assert_eq!(client.get_merchant_key(&merchant), key);

    assert!(
        !events.is_empty(),
        "No events captured immediately after key set!"
    );

    let (event_contract_id, _topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(event_contract_id, contract_id.clone());

    let data_map: Map<Symbol, Val> = data.try_into_val(&env).expect("Data should be a Map");

    let merchant_val = data_map
        .get(Symbol::new(&env, "merchant"))
        .expect("Should have merchant field");
    let key_val = data_map
        .get(Symbol::new(&env, "key"))
        .expect("Should have key field");

    let merchant_in_event: Address = merchant_val.try_into_val(&env).unwrap();
    let key_in_event: BytesN<32> = key_val.try_into_val(&env).unwrap();

    assert_eq!(merchant_in_event, merchant.clone());
    assert_eq!(key_in_event, key.clone());
}

#[test]
fn test_update_merchant_key() {
    let (env, client, _contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let key1 = BytesN::from_array(&env, &[0u8; 32]);
    client.set_merchant_key(&merchant, &key1);
    assert_eq!(client.get_merchant_key(&merchant), key1);

    let key2 = BytesN::from_array(&env, &[1u8; 32]);
    client.set_merchant_key(&merchant, &key2);
    assert_eq!(client.get_merchant_key(&merchant), key2);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_get_non_existent_key() {
    let (env, client, _contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    client.get_merchant_key(&merchant);
}
