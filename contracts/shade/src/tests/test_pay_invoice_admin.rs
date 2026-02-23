#![cfg(test)]

use crate::shade::{Shade, ShadeClient};
use crate::types::{InvoiceStatus, Role};
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{token, Address, Env, Map, String, Symbol, TryIntoVal, Val};

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

fn create_test_token(env: &Env) -> Address {
    let token_admin = Address::generate(env);
    env.register_stellar_asset_contract_v2(token_admin).address()
}

/// Sets up a merchant, token, invoice and funds the shade contract
/// Returns (merchant_address, token_address, invoice_id)
fn setup_funded_invoice(
    env: &Env,
    client: &ShadeClient,
    contract_id: &Address,
    admin: &Address,
    amount: i128,
    fee: i128,
) -> (Address, Address, u64) {
    let merchant = Address::generate(env);
    client.register_merchant(&merchant);

    let token = create_test_token(env);

    // Add token and set fee
    client.add_accepted_token(admin, &token);
    client.set_fee(admin, &token, &fee);

    // Create invoice
    let description = String::from_str(env, "Test Invoice");
    let invoice_id = client.create_invoice(&merchant, &description, &amount, &token);

    // Fund the shade contract with the invoice amount
    token::StellarAssetClient::new(env, &token).mint(contract_id, &amount);

    (merchant, token, invoice_id)
}

fn assert_invoice_paid_event(
    env: &Env,
    contract_id: &Address,
    expected_invoice_id: u64,
    expected_payer: &Address,
    expected_merchant: &Address,
    expected_amount: i128,
    expected_fee: i128,
    expected_token: &Address,
) {
    let events = env.events().all();
    assert!(events.len() > 0, "No events emitted");

    let (event_contract_id, _topics, data) = events.get(events.len() - 1).unwrap();
    assert_eq!(&event_contract_id, contract_id);

    let data_map: Map<Symbol, Val> = data.try_into_val(env).unwrap();

    let invoice_id_in_event: u64 = data_map
        .get(Symbol::new(env, "invoice_id"))
        .unwrap()
        .try_into_val(env)
        .unwrap();
    let payer_in_event: Address = data_map
        .get(Symbol::new(env, "payer"))
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
    let fee_in_event: i128 = data_map
        .get(Symbol::new(env, "fee_amount"))
        .unwrap()
        .try_into_val(env)
        .unwrap();
    let token_in_event: Address = data_map
        .get(Symbol::new(env, "token"))
        .unwrap()
        .try_into_val(env)
        .unwrap();

    assert_eq!(invoice_id_in_event, expected_invoice_id);
    assert_eq!(payer_in_event, expected_payer.clone());
    assert_eq!(merchant_in_event, expected_merchant.clone());
    assert_eq!(amount_in_event, expected_amount);
    assert_eq!(fee_in_event, expected_fee);
    assert_eq!(token_in_event, expected_token.clone());
}

// ── 1. Admin successfully pays invoice ───────────────────────────────────────

#[test]
fn test_pay_invoice_admin_by_admin_success() {
    let (env, client, contract_id, admin) = setup_test();
    let amount: i128 = 1_000;
    let fee: i128 = 100;
    let (merchant, token, invoice_id) =
        setup_funded_invoice(&env, &client, &contract_id, &admin, amount, fee);

    let payer = Address::generate(&env);
    client.pay_invoice_admin(&admin, &payer, &invoice_id);

    // Invoice status updated to Paid
    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.status, InvoiceStatus::Paid);
    assert_eq!(invoice.payer, Some(payer.clone()));
    assert!(invoice.date_paid.is_some());

    // Merchant receives net amount
    let token_client = token::TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&merchant), amount - fee);

    // Fee stays in Shade contract
    assert_eq!(token_client.balance(&contract_id), fee);

    assert_invoice_paid_event(
        &env,
        &contract_id,
        invoice_id,
        &payer,
        &merchant,
        amount,
        fee,
        &token,
    );
}

// ── 2. Manager successfully pays invoice ─────────────────────────────────────

#[test]
fn test_pay_invoice_admin_by_manager_success() {
    let (env, client, contract_id, admin) = setup_test();
    let amount: i128 = 2_000;
    let fee: i128 = 200;
    let (merchant, token, invoice_id) =
        setup_funded_invoice(&env, &client, &contract_id, &admin, amount, fee);

    let manager = Address::generate(&env);
    client.grant_role(&admin, &manager, &Role::Manager);

    let payer = Address::generate(&env);
    client.pay_invoice_admin(&manager, &payer, &invoice_id);

    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.status, InvoiceStatus::Paid);
    assert_eq!(invoice.payer, Some(payer.clone()));

    let token_client = token::TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&merchant), amount - fee);
    assert_eq!(token_client.balance(&contract_id), fee);

    assert_invoice_paid_event(
        &env,
        &contract_id,
        invoice_id,
        &payer,
        &merchant,
        amount,
        fee,
        &token,
    );
}

// ── 3. Zero fee — full amount goes to merchant ────────────────────────────────

#[test]
fn test_pay_invoice_admin_zero_fee() {
    let (env, client, contract_id, admin) = setup_test();
    let amount: i128 = 500;
    let fee: i128 = 0;
    let (merchant, token, invoice_id) =
        setup_funded_invoice(&env, &client, &contract_id, &admin, amount, fee);

    let payer = Address::generate(&env);
    client.pay_invoice_admin(&admin, &payer, &invoice_id);

    let token_client = token::TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&merchant), amount);
    assert_eq!(token_client.balance(&contract_id), 0);
}

// ── 4. Unauthorized caller is rejected ───────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_pay_invoice_admin_unauthorized_fails() {
    let (env, client, contract_id, admin) = setup_test();
    let (_, _, invoice_id) =
        setup_funded_invoice(&env, &client, &contract_id, &admin, 1_000, 100);

    let random = Address::generate(&env);
    let payer = Address::generate(&env);
    client.pay_invoice_admin(&random, &payer, &invoice_id);
}

// ── 5. Paying already paid invoice is rejected ───────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #15)")]
fn test_pay_invoice_admin_already_paid_fails() {
    let (env, client, contract_id, admin) = setup_test();
    let amount: i128 = 1_000;
    let fee: i128 = 100;
    let (_, token, invoice_id) =
        setup_funded_invoice(&env, &client, &contract_id, &admin, amount, fee);

    let payer = Address::generate(&env);

    // First payment succeeds
    client.pay_invoice_admin(&admin, &payer, &invoice_id);

    // Fund again and attempt second payment — must panic
    token::StellarAssetClient::new(&env, &token).mint(&contract_id, &amount);
    client.pay_invoice_admin(&admin, &payer, &invoice_id);
}

// ── 6. Non-existent invoice is rejected ──────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_pay_invoice_admin_invoice_not_found_fails() {
    let (env, client, _contract_id, admin) = setup_test();
    let payer = Address::generate(&env);
    client.pay_invoice_admin(&admin, &payer, &999);
}

// ── 7. Paused contract rejects payment ───────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_pay_invoice_admin_when_paused_fails() {
    let (env, client, contract_id, admin) = setup_test();
    let (_, _, invoice_id) =
        setup_funded_invoice(&env, &client, &contract_id, &admin, 1_000, 100);

    client.pause(&admin);

    let payer = Address::generate(&env);
    client.pay_invoice_admin(&admin, &payer, &invoice_id);
}