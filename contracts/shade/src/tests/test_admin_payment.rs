#![cfg(test)]

use crate::shade::{Shade, ShadeClient};
use crate::types::{InvoiceStatus, Role};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String};

fn setup_invoice_test() -> (
    Env,
    ShadeClient<'static>,
    Address,
    Address,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let manager = Address::generate(&env);
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);

    // Create token
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    // Add accepted token
    client.add_accepted_token(&admin, &token);

    // Set fee
    let fee: i128 = 100;
    client.set_fee(&admin, &token, &fee);

    (env, client, admin, manager, merchant, payer, token)
}

#[test]
fn test_invoice_state_validation() {
    let (env, client, _admin, _manager, merchant, _payer, token) = setup_invoice_test();

    // Register merchant
    client.register_merchant(&merchant);

    // Create invoice
    let invoice_id = client.create_invoice(
        &merchant,
        &String::from_str(&env, "Test Invoice"),
        &1000,
        &token,
    );

    // Verify initial state
    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
    assert_eq!(invoice.payer, None);
    assert_eq!(invoice.date_paid, None);
}

#[test]
fn test_multiple_invoices_independent() {
    let (env, client, _admin, _manager, merchant, _payer, token) = setup_invoice_test();

    // Register merchant
    client.register_merchant(&merchant);

    // Create multiple invoices
    let id_1 = client.create_invoice(
        &merchant,
        &String::from_str(&env, "Invoice 1"),
        &1000,
        &token,
    );
    let id_2 = client.create_invoice(
        &merchant,
        &String::from_str(&env, "Invoice 2"),
        &2000,
        &token,
    );

    // Set second to Paid via storage manipulation
    use crate::types::DataKey;
    let mut inv_2 = client.get_invoice(&id_2);
    inv_2.status = InvoiceStatus::Paid;
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::Invoice(id_2), &inv_2);
    });

    // Verify first is still Pending
    assert_eq!(client.get_invoice(&id_1).status, InvoiceStatus::Pending);
    assert_eq!(client.get_invoice(&id_2).status, InvoiceStatus::Paid);
}

#[test]
fn test_fee_preservation() {
    let (env, client, admin, _manager, merchant, _payer, token) = setup_invoice_test();

    // Register merchant
    client.register_merchant(&merchant);

    // Set custom fee
    let fee = 250i128;
    client.set_fee(&admin, &token, &fee);

    // Create invoice
    let invoice_id = client.create_invoice(
        &merchant,
        &String::from_str(&env, "Test Invoice"),
        &1000,
        &token,
    );

    // Verify fee and invoice data
    assert_eq!(client.get_fee(&token), fee);
    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.amount, 1000);
}

#[test]
fn test_manager_role_grant_and_revoke() {
    let (_env, client, admin, manager, _merchant, _payer, _token) = setup_invoice_test();

    // Grant manager role
    client.grant_role(&admin, &manager, &Role::Manager);
    assert!(client.has_role(&manager, &Role::Manager));

    // Revoke manager role
    client.revoke_role(&admin, &manager, &Role::Manager);
    assert!(!client.has_role(&manager, &Role::Manager));
}

#[test]
fn test_contract_pause_and_unpause() {
    let (env, client, admin, _manager, merchant, _payer, token) = setup_invoice_test();

    // Register merchant
    client.register_merchant(&merchant);

    // Pause contract
    client.pause(&admin);
    assert!(client.is_paused());

    // Unpause contract
    client.unpause(&admin);
    assert!(!client.is_paused());

    // Should be able to create invoices after unpause
    let invoice_id = client.create_invoice(
        &merchant,
        &String::from_str(&env, "Post-unpause invoice"),
        &500,
        &token,
    );
    assert!(invoice_id > 0);
}
