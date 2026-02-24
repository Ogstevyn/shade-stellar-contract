#![cfg(test)]

use crate::shade::{Shade, ShadeClient};
use crate::types::{InvoiceStatus, Role};
use soroban_sdk::testutils::{Address as _, Events as_};
use soroban_sdk::{Address, BytesN, Env, Map, String, Symbol, TryIntoVal, Val};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn setup_test() -> (Env, ShadeClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, contract_id, admin)
}

/// Generates a fresh 32-byte nonce
fn generate_nonce(env: &Env) -> BytesN<32> {
    let bytes: [u8; 32] = [1u8; 32];
    BytesN::from_array(env, &bytes)
}

/// Generates a different nonce to test replay protection
fn generate_nonce_2(env: &Env) -> BytesN<32> {
    let bytes: [u8; 32] = [2u8; 32];
    BytesN::from_array(env, &bytes)
}

/// Generates a mock 64-byte signature
/// In tests with mock_all_auths(), crypto verification is bypassed
fn generate_signature(env: &Env) -> BytesN<64> {
    let bytes: [u8; 64] = [0u8; 64];
    BytesN::from_array(env, &bytes)
}

/// Sets up a registered merchant with a key stored
fn setup_merchant_with_key(
    env: &Env,
    client: &ShadeClient,
) -> (Address, BytesN<32>) {
    let merchant = Address::generate(env);
    client.register_merchant(&merchant);

    let key_bytes: [u8; 32] = [9u8; 32];
    let key = BytesN::from_array(env, &key_bytes);
    client.set_merchant_key(&merchant, &key);

    (merchant, key)
}

fn assert_latest_invoice_event(
    env: &Env,
    contract_id: &Address,
    expected_invoice_id: u64,
    expected_merchant: &Address,
    expected_amount: i128,
    expected_token: &Address,
) {
    let events = env.events().all();
    assert!(events.len() > 0, "No events captured for invoice!");

    let (event_contract_id, _topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(&event_contract_id, contract_id);

    let data_map: Map<Symbol, Val> = data.try_into_val(env).unwrap();

    let invoice_id_in_event: u64 = data_map
        .get(Symbol::new(env, "invoice_id"))
        .unwrap()
        .try_into_val(env)
        .unwrap();
    let merchant_in_event: Address = data_map
        .get(Symbol::new(env, "merchant"))
        .unwrap()
        .try_into_val(env)
        .unwrap();
    let amount_in_event: i128 = data_map
        .get(Symbol::new(env, "amount"))
        .unwrap()
        .try_into_val(env)
        .unwrap();
    let token_in_event: Address = data_map
        .get(Symbol::new(env, "token"))
        .unwrap()
        .try_into_val(env)
        .unwrap();

    assert_eq!(invoice_id_in_event, expected_invoice_id);
    assert_eq!(merchant_in_event, expected_merchant.clone());
    assert_eq!(amount_in_event, expected_amount);
    assert_eq!(token_in_event, expected_token.clone());
}

// ── 1. Manager can create invoice on behalf of merchant ───────────────────────

#[test]
fn test_create_invoice_signed_by_manager_success() {
    let (env, client, contract_id, admin) = setup_test();
    let (merchant, _key) = setup_merchant_with_key(&env, &client);

    let manager = Address::generate(&env);
    client.grant_role(&admin, &manager, &Role::Manager);

    let token = Address::generate(&env);
    let description = String::from_str(&env, "Manager signed invoice");
    let amount: i128 = 1_500;
    let nonce = generate_nonce(&env);
    let signature = generate_signature(&env);

    let invoice_id = client.create_invoice_signed(
        &manager,
        &merchant,
        &description,
        &amount,
        &token,
        &nonce,
        &signature,
    );

    assert_eq!(invoice_id, 1);

    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.id, 1);
    assert_eq!(invoice.amount, amount);
    assert_eq!(invoice.token, token);
    assert_eq!(invoice.description, description);
    assert_eq!(invoice.status, InvoiceStatus::Pending);

    assert_latest_invoice_event(&env, &contract_id, invoice_id, &merchant, amount, &token);
}

// ── 2. Admin can create invoice on behalf of merchant ─────────────────────────

#[test]
fn test_create_invoice_signed_by_admin_success() {
    let (env, client, contract_id, admin) = setup_test();
    let (merchant, _key) = setup_merchant_with_key(&env, &client);

    let token = Address::generate(&env);
    let description = String::from_str(&env, "Admin signed invoice");
    let amount: i128 = 2_000;
    let nonce = generate_nonce(&env);
    let signature = generate_signature(&env);

    // Admin has implicit role via has_role check in access_control
    let invoice_id = client.create_invoice_signed(
        &admin,
        &merchant,
        &description,
        &amount,
        &token,
        &nonce,
        &signature,
    );

    assert_eq!(invoice_id, 1);

    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.amount, amount);
    assert_eq!(invoice.status, InvoiceStatus::Pending);

    assert_latest_invoice_event(&env, &contract_id, invoice_id, &merchant, amount, &token);
}

// ── 3. Nonce is invalidated after use (replay attack prevention) ──────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #14)")]
fn test_create_invoice_signed_nonce_replay_fails() {
    let (env, client, _contract_id, admin) = setup_test();
    let (merchant, _key) = setup_merchant_with_key(&env, &client);

    let manager = Address::generate(&env);
    client.grant_role(&admin, &manager, &Role::Manager);

    let token = Address::generate(&env);
    let description = String::from_str(&env, "Replay test");
    let amount: i128 = 500;
    let nonce = generate_nonce(&env);
    let signature = generate_signature(&env);

    // First call succeeds
    client.create_invoice_signed(
        &manager,
        &merchant,
        &description,
        &amount,
        &token,
        &nonce,
        &signature,
    );

    // Second call with same nonce must panic with NonceAlreadyUsed (#14)
    client.create_invoice_signed(
        &manager,
        &merchant,
        &description,
        &amount,
        &token,
        &nonce,
        &signature,
    );
}

// ── 4. Different nonces work independently ────────────────────────────────────

#[test]
fn test_create_invoice_signed_different_nonces_succeed() {
    let (env, client, _contract_id, admin) = setup_test();
    let (merchant, _key) = setup_merchant_with_key(&env, &client);

    let manager = Address::generate(&env);
    client.grant_role(&admin, &manager, &Role::Manager);

    let token = Address::generate(&env);
    let description = String::from_str(&env, "Multi nonce test");
    let amount: i128 = 300;

    let id1 = client.create_invoice_signed(
        &manager,
        &merchant,
        &description,
        &amount,
        &token,
        &generate_nonce(&env),
        &generate_signature(&env),
    );

    let id2 = client.create_invoice_signed(
        &manager,
        &merchant,
        &description,
        &amount,
        &token,
        &generate_nonce_2(&env),
        &generate_signature(&env),
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

// ── 5. Unauthorized caller is rejected ───────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_create_invoice_signed_unauthorized_caller_fails() {
    let (env, client, _contract_id, _admin) = setup_test();
    let (merchant, _key) = setup_merchant_with_key(&env, &client);

    let random = Address::generate(&env);
    let token = Address::generate(&env);
    let description = String::from_str(&env, "Unauthorized test");

    client.create_invoice_signed(
        &random,
        &merchant,
        &description,
        &500,
        &token,
        &generate_nonce(&env),
        &generate_signature(&env),
    );
}

// ── 6. Unregistered merchant is rejected ─────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_create_invoice_signed_unregistered_merchant_fails() {
    let (env, client, _contract_id, admin) = setup_test();

    let manager = Address::generate(&env);
    client.grant_role(&admin, &manager, &Role::Manager);

    let unregistered_merchant = Address::generate(&env);
    let token = Address::generate(&env);
    let description = String::from_str(&env, "Unregistered merchant");

    client.create_invoice_signed(
        &manager,
        &unregistered_merchant,
        &description,
        &500,
        &token,
        &generate_nonce(&env),
        &generate_signature(&env),
    );
}

// ── 7. Invalid amount is rejected ────────────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_create_invoice_signed_invalid_amount_fails() {
    let (env, client, _contract_id, admin) = setup_test();
    let (merchant, _key) = setup_merchant_with_key(&env, &client);

    let manager = Address::generate(&env);
    client.grant_role(&admin, &manager, &Role::Manager);

    let token = Address::generate(&env);
    let description = String::from_str(&env, "Zero amount test");

    client.create_invoice_signed(
        &manager,
        &merchant,
        &description,
        &0,
        &token,
        &generate_nonce(&env),
        &generate_signature(&env),
    );
}

// ── 8. Paused contract rejects all calls ─────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_create_invoice_signed_when_paused_fails() {
    let (env, client, _contract_id, admin) = setup_test();
    let (merchant, _key) = setup_merchant_with_key(&env, &client);

    let manager = Address::generate(&env);
    client.grant_role(&admin, &manager, &Role::Manager);

    client.pause(&admin);

    let token = Address::generate(&env);
    let description = String::from_str(&env, "Paused test");

    client.create_invoice_signed(
        &manager,
        &merchant,
        &description,
        &500,
        &token,
        &generate_nonce(&env),
        &generate_signature(&env),
    );
}