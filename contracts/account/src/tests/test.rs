#![cfg(test)]

use crate::account::MerchantAccount;
use crate::account::MerchantAccountClient;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(MerchantAccount, ());
    let client = MerchantAccountClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    let manager = Address::generate(&env);
    let merchant_id = 1;
    client.initialize(&merchant, &manager, &merchant_id);
    assert_eq!(client.get_merchant(), merchant);
}

#[should_panic(expected = "HostError: Error(Contract, #1)")]
#[test]
fn test_initialize_twice() {
    let env = Env::default();
    let contract_id = env.register(MerchantAccount, ());
    let client = MerchantAccountClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    let manager = Address::generate(&env);
    let merchant_id = 1;
    client.initialize(&merchant, &manager, &merchant_id);
    client.initialize(&merchant, &manager, &merchant_id);
}

#[should_panic(expected = "HostError: Error(Contract, #2)")]
#[test]
fn test_get_merchant_not_initialized() {
    let env = Env::default();
    let contract_id = env.register(MerchantAccount, ());
    let client = MerchantAccountClient::new(&env, &contract_id);

    client.get_merchant();
}
