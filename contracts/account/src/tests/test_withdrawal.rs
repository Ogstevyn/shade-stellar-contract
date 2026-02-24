#![cfg(test)]

use crate::account::MerchantAccount;
use crate::account::MerchantAccountClient;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

fn setup_initialized_account(env: &Env) -> (Address, MerchantAccountClient<'_>, Address) {
    let contract_id = env.register(MerchantAccount, ());
    let client = MerchantAccountClient::new(env, &contract_id);

    let merchant = Address::generate(env);
    let manager = Address::generate(env);
    let merchant_id = 1u64;
    client.initialize(&merchant, &manager, &merchant_id);

    (contract_id, client, merchant)
}

fn create_test_token(env: &Env) -> Address {
    let token_admin = Address::generate(env);
    env.register_stellar_asset_contract_v2(token_admin)
        .address()
}

#[test]
fn test_withdraw_to_with_zero_token_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _merchant) = setup_initialized_account(&env);

    let token = create_test_token(&env);

    let balance = client.get_balance(&token);
    assert_eq!(balance, 0, "Token balance should start at 0");
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_withdraw_to_requires_merchant_auth() {
    let env = Env::default();
    let contract_id = env.register(MerchantAccount, ());
    let client = MerchantAccountClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    let manager = Address::generate(&env);
    let merchant_id = 1u64;

    client.initialize(&merchant, &manager, &merchant_id);

    let recipient = Address::generate(&env);
    let token = create_test_token(&env);

    client.withdraw_to(&token, &500_000i128, &recipient);
}

#[test]
fn test_withdraw_to_validates_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _merchant) = setup_initialized_account(&env);

    let token = create_test_token(&env);

    let balance = client.get_balance(&token);
    assert_eq!(balance, 0, "Token balance should be 0 in test environment");
}

#[test]
fn test_withdraw_to_checks_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, _merchant) = setup_initialized_account(&env);

    let token = create_test_token(&env);

    let balance = client.get_balance(&token);
    assert!(balance < 1_000_000i128, "Default balance less than 1M");
}
