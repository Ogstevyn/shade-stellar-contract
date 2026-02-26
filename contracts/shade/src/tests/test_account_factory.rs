#![cfg(test)]

use crate::shade::{Shade, ShadeClient};
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Address, BytesN, Env, Map, Symbol, TryIntoVal, Val, Vec};

// const ACCOUNT_WASM: &[u8] =
//     include_bytes!("../../../../target/wasm32-unknown-unknown/release/account.wasm");

fn setup() -> (Env, ShadeClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, client, contract_id)
}

fn assert_latest_merchant_account_deployed_event(
    env: &Env,
    contract_id: &Address,
    expected_merchant: &Address,
    expected_contract: &Address,
    expected_timestamp: u64,
) {
    let events = env.events().all();
    assert!(!events.is_empty());

    // Find the most recent merchant_account_deployed_event
    let mut found = false;
    for i in (0..events.len()).rev() {
        let (event_contract_id_i, topics_i, data_i) = events.get(i).unwrap();
        if topics_i.len() == 1 {
            let event_name: Symbol = topics_i.get(0).unwrap().try_into_val(env).unwrap();
            if event_name == Symbol::new(env, "merchant_account_deployed_event") {
                assert_eq!(event_contract_id_i, contract_id.clone());
                let data_map: Map<Symbol, Val> = data_i.try_into_val(env).unwrap();
                let merchant_val = data_map.get(Symbol::new(env, "merchant")).unwrap();
                let contract_val = data_map.get(Symbol::new(env, "contract")).unwrap();
                let timestamp_val = data_map.get(Symbol::new(env, "timestamp")).unwrap();

                let merchant_in_event: Address = merchant_val.try_into_val(env).unwrap();
                let contract_in_event: Address = contract_val.try_into_val(env).unwrap();
                let timestamp_in_event: u64 = timestamp_val.try_into_val(env).unwrap();

                assert_eq!(merchant_in_event, expected_merchant.clone());
                assert_eq!(contract_in_event, expected_contract.clone());
                assert_eq!(timestamp_in_event, expected_timestamp);
                found = true;
                break;
            }
        }
    }
    assert!(found, "merchant_account_deployed_event not found in events");
}

#[test]
fn test_deploy_account_in_isolation_and_initialize() {
    let (env, _client, contract_id) = setup();

    let merchant = Address::generate(&env);

    // Use test helper to register and initialize a MerchantAccount directly (avoids deployer/wasm complexity)
    let expected_timestamp = env.ledger().timestamp();
    let deployed = {
        let acct_id = env.register(account::account::MerchantAccount, ());
        let acct_client = account::account::MerchantAccountClient::new(&env, &acct_id);
        acct_client.initialize(&merchant, &contract_id, &1_u64);
        // publish event as the factory would (emit from Shade contract context)
        env.as_contract(&contract_id, || {
            crate::events::publish_merchant_account_deployed_event(
                &env,
                merchant.clone(),
                acct_id.clone(),
                env.ledger().timestamp(),
            );
        });
        acct_id
    };

    assert_latest_merchant_account_deployed_event(
        &env,
        &contract_id,
        &merchant,
        &deployed,
        expected_timestamp,
    );

    // Verify the deployed account is initialized and returns the merchant
    let merchant_from_account: Address = env.as_contract(&deployed, || {
        env.storage()
            .persistent()
            .get(&account::types::DataKey::Merchant)
            .unwrap()
    });
    assert_eq!(merchant_from_account, merchant);
}

#[test]
fn test_register_merchant_integration_and_uniqueness() {
    let (env, client, contract_id) = setup();

    let merchant_a = Address::generate(&env);
    let merchant_b = Address::generate(&env);

    // Register merchants in shade
    client.register_merchant(&merchant_a);
    client.register_merchant(&merchant_b);

    // For testing integration we register account instances directly and simulate factory behavior
    let deployed_a = {
        let acct_id = env.register(account::account::MerchantAccount, ());
        let acct_client = account::account::MerchantAccountClient::new(&env, &acct_id);
        acct_client.initialize(&merchant_a, &contract_id, &1_u64);
        env.as_contract(&contract_id, || {
            crate::events::publish_merchant_account_deployed_event(
                &env,
                merchant_a.clone(),
                acct_id.clone(),
                env.ledger().timestamp(),
            );
        });
        acct_id
    };

    let deployed_b = {
        let acct_id = env.register(account::account::MerchantAccount, ());
        let acct_client = account::account::MerchantAccountClient::new(&env, &acct_id);
        acct_client.initialize(&merchant_b, &contract_id, &2_u64);
        env.as_contract(&contract_id, || {
            crate::events::publish_merchant_account_deployed_event(
                &env,
                merchant_b.clone(),
                acct_id.clone(),
                env.ledger().timestamp(),
            );
        });
        acct_id
    };

    // Ensure uniqueness
    assert_ne!(deployed_a, deployed_b);

    // Link accounts in shade (simulate on-chain linkage)
    client.set_merchant_account(&merchant_a, &deployed_a);
    client.set_merchant_account(&merchant_b, &deployed_b);

    // Verify shade reports the linked accounts
    let linked_a = client.get_merchant_account(&1u64);
    let linked_b = client.get_merchant_account(&2u64);
    assert_eq!(linked_a, deployed_a);
    assert_eq!(linked_b, deployed_b);

    // Verify deployed accounts are initialized correctly
    let mac_a_merchant: Address = env.as_contract(&deployed_a, || {
        env.storage()
            .persistent()
            .get(&account::types::DataKey::Merchant)
            .unwrap()
    });
    let mac_b_merchant: Address = env.as_contract(&deployed_b, || {
        env.storage()
            .persistent()
            .get(&account::types::DataKey::Merchant)
            .unwrap()
    });
    assert_eq!(mac_a_merchant, merchant_a);
    assert_eq!(mac_b_merchant, merchant_b);
}
