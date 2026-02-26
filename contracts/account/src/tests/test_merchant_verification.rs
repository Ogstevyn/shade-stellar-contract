#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Env, IntoVal,
};

use crate::account::MerchantAccount;
use crate::events::AccountVerified;

// ── helpers ──────────────────────────────────────────────────────────────────

fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(MerchantAccount, ());
    let client = MerchantAccountClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    let manager = Address::generate(&env);

    client.initialize(&merchant, &manager, &1u64);

    (env, contract_id, merchant, manager)
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// A newly initialized account must not be verified.
#[test]
fn test_initial_state_is_unverified() {
    let (env, contract_id, _, _) = setup();
    let client = MerchantAccountClient::new(&env, &contract_id);

    assert!(
        !client.is_verified_account(),
        "account should be unverified after initialization"
    );
}

/// The manager can verify an account; afterwards `is_verified_account` returns true
/// and an `AccountVerified` event is emitted.
#[test]
fn test_successful_verification() {
    let (env, contract_id, _, _) = setup();
    let client = MerchantAccountClient::new(&env, &contract_id);

    client.verify_account();

    // Status check
    assert!(
        client.is_verified_account(),
        "account should be verified after verify_account"
    );

    // Event check — find the AccountVerified event in the emitted list
    let events = env.events().all();
    let verified_event = events.iter().find(|(contract, _topics, _data)| {
        *contract == contract_id
    });

    assert!(
        verified_event.is_some(),
        "AccountVerified event should have been emitted"
    );

    // Confirm the event data carries a timestamp (u64)
    let (_, _topics, data) = verified_event.unwrap();
    let _: u64 = data.into_val(&env); // will panic if type doesn't match
}

/// Calling `verify_account` from a non-manager address must panic.
#[test]
#[should_panic]
fn test_unauthorized_verification_panics() {
    let env = Env::default();
    // Do NOT mock_all_auths — we want real auth enforcement
    let contract_id = env.register(MerchantAccount, ());
    let client = MerchantAccountClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    let manager = Address::generate(&env);

    // Initialize with mock auths just for setup
    env.mock_all_auths();
    client.initialize(&merchant, &manager, &1u64);

    // Clear mocked auths so the next call uses real auth
    // (creating a fresh env simulates this cleanly)
    let env2 = Env::default();
    let contract_id2 = env2.register(MerchantAccount, ());
    let client2 = MerchantAccountClient::new(&env2, &contract_id2);

    let merchant2 = Address::generate(&env2);
    let manager2 = Address::generate(&env2);
    let impostor = Address::generate(&env2);

    env2.mock_all_auths();
    client2.initialize(&merchant2, &manager2, &2u64);

    // Only authorize the impostor, not the manager
    env2.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &impostor,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id2,
            fn_name: "verify_account",
            args: soroban_sdk::vec![&env2].into(),
            sub_invokes: &[],
        },
    }]);

    // This should panic because impostor != manager
    client2.verify_account();
}

/// Calling `verify_account` twice must not panic — the operation is idempotent.
#[test]
fn test_verify_account_is_idempotent() {
    let (env, contract_id, _, _) = setup();
    let client = MerchantAccountClient::new(&env, &contract_id);

    client.verify_account();
    client.verify_account(); // second call must not panic

    assert!(
        client.is_verified_account(),
        "account should still be verified after second call"
    );
}