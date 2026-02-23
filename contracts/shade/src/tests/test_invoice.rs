#![cfg(test)]

use crate::shade::{Shade, ShadeClient};
use crate::types::InvoiceStatus;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Address, Env, Map, String, Symbol, TryIntoVal, Val};

fn setup_test() -> (Env, ShadeClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, contract_id, admin)
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

    let invoice_id_val = data_map.get(Symbol::new(env, "invoice_id")).unwrap();
    let merchant_val = data_map.get(Symbol::new(env, "merchant")).unwrap();
    let amount_val = data_map.get(Symbol::new(env, "amount")).unwrap();
    let token_val = data_map.get(Symbol::new(env, "token")).unwrap();

    let invoice_id_in_event: u64 = invoice_id_val.try_into_val(env).unwrap();
    let merchant_in_event: Address = merchant_val.try_into_val(env).unwrap();
    let amount_in_event: i128 = amount_val.try_into_val(env).unwrap();
    let token_in_event: Address = token_val.try_into_val(env).unwrap();

    assert_eq!(invoice_id_in_event, expected_invoice_id);
    assert_eq!(merchant_in_event, expected_merchant.clone());
    assert_eq!(amount_in_event, expected_amount);
    assert_eq!(token_in_event, expected_token.clone());
}

#[test]
fn test_create_and_get_invoice_success() {
    let (env, client, contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let token = Address::generate(&env);
    let description = String::from_str(&env, "Test Invoice");
    let amount: i128 = 1000;

    let invoice_id = client.create_invoice(&merchant, &description, &amount, &token);
    assert_eq!(invoice_id, 1);

    assert_latest_invoice_event(&env, &contract_id, invoice_id, &merchant, amount, &token);

    let invoice = client.get_invoice(&invoice_id);

    assert_eq!(invoice.id, 1);
    assert_eq!(invoice.merchant_id, 1);
    assert_eq!(invoice.amount, amount);
    assert_eq!(invoice.token, token);
    assert_eq!(invoice.description, description);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
}

#[test]
fn test_create_multiple_invoices() {
    let (env, client, _contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let token1 = Address::generate(&env);
    let token2 = Address::generate(&env);

    let id1 = client.create_invoice(
        &merchant,
        &String::from_str(&env, "Invoice 1"),
        &1000,
        &token1,
    );
    let id2 = client.create_invoice(
        &merchant,
        &String::from_str(&env, "Invoice 2"),
        &2000,
        &token2,
    );
    let id3 = client.create_invoice(
        &merchant,
        &String::from_str(&env, "Invoice 3"),
        &500,
        &token1,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

#[should_panic(expected = "HostError: Error(Contract, #8)")]
#[test]
fn test_get_invoice_not_found() {
    let (_env, client, _contract_id, _admin) = setup_test();
    client.get_invoice(&999);
}

#[should_panic(expected = "HostError: Error(Contract, #1)")]
#[test]
fn test_create_invoice_unregistered_merchant() {
    let (env, client, _contract_id, _admin) = setup_test();

    let unregistered_merchant = Address::generate(&env);
    let token = Address::generate(&env);
    let description = String::from_str(&env, "Test Invoice");
    let amount: i128 = 1000;

    client.create_invoice(&unregistered_merchant, &description, &amount, &token);
}

#[should_panic(expected = "HostError: Error(Contract, #7)")]
#[test]
fn test_create_invoice_invalid_amount() {
    let (env, client, _contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let token = Address::generate(&env);
    let description = String::from_str(&env, "Test Invoice");
    let amount: i128 = 0;

    client.create_invoice(&merchant, &description, &amount, &token);
}

// Add simplistic mock token test for refunds or at least test the validations
// Since refunding requires a token transfer and we are not setting up a full token contract mock here,
// we will just test the validations up to the require_auth or error points, 
// or maybe skip full token integration if it's too complex for this file.
// Alternatively, we can use the default token implementation from Soroban SDK if available.

#[test]
#[should_panic(expected = "HostError: Error(Contract, #13)")]
fn test_refund_invalid_status() {
    let (env, client, _contract_id, _admin) = setup_test();

    let merchant = Address::generate(&env);
    client.register_merchant(&merchant);

    let token = Address::generate(&env);
    let description = String::from_str(&env, "Test Invoice");
    let amount: i128 = 1000;

    let invoice_id = client.create_invoice(&merchant, &description, &amount, &token);
    
    // Status is pending, should panic with InvalidInvoiceStatus
    client.refund_invoice_partial(&invoice_id, &500);
}

