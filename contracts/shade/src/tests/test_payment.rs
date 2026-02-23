#![cfg(test)]

use crate::shade::{Shade, ShadeClient};
use crate::types::InvoiceStatus;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{token, Address, Env, String};

fn setup_test_with_payment() -> (Env, ShadeClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    // Register Shade contract
    let shade_contract_id = env.register(Shade, ());
    let shade_client = ShadeClient::new(&env, &shade_contract_id);

    // Initialize with admin
    let admin = Address::generate(&env);
    shade_client.initialize(&admin);

    // Create and register token
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone());

    // Add token as accepted
    shade_client.add_accepted_token(&admin, &token.address());

    // Set fee to 500 bps (5%)
    shade_client.set_fee(&admin, &token.address(), &500);

    (env, shade_client, shade_contract_id, admin, token.address())
}

#[test]
fn test_successful_payment_with_fee() {
    let (env, shade_client, shade_contract_id, _admin, token) = setup_test_with_payment();

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // Create merchant account (using a regular address as mock)
    let merchant_account = Address::generate(&env);
    shade_client.set_merchant_account(&merchant, &merchant_account);

    // Create invoice for 1000 units
    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &1000, &token);

    // Create customer and mint tokens
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&customer, &1000);

    // Customer pays invoice
    shade_client.pay_invoice(&customer, &invoice_id);

    // Verify balances
    let token_balance_client = token::TokenClient::new(&env, &token);
    let shade_balance = token_balance_client.balance(&shade_contract_id);
    let merchant_balance = token_balance_client.balance(&merchant_account);

    assert_eq!(shade_balance, 50); // 5% fee = 50 units
    assert_eq!(merchant_balance, 950); // 95% = 950 units

    // Verify invoice status
    let invoice = shade_client.get_invoice(&invoice_id);
    assert_eq!(invoice.status, InvoiceStatus::Paid);
    assert_eq!(invoice.payer, Some(customer));
    assert!(invoice.date_paid.is_some());
}

#[test]
fn test_payment_with_zero_fee() {
    let (env, shade_client, shade_contract_id, admin, token) = setup_test_with_payment();

    // Set fee to 0 bps (0%)
    shade_client.set_fee(&admin, &token, &0);

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // Create merchant account
    let merchant_account = Address::generate(&env);
    shade_client.set_merchant_account(&merchant, &merchant_account);

    // Create invoice for 1000 units
    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &1000, &token);

    // Create customer and mint tokens
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&customer, &1000);

    // Customer pays invoice
    shade_client.pay_invoice(&customer, &invoice_id);

    // Verify balances
    let token_balance_client = token::TokenClient::new(&env, &token);
    let shade_balance = token_balance_client.balance(&shade_contract_id);
    let merchant_balance = token_balance_client.balance(&merchant_account);

    assert_eq!(shade_balance, 0); // 0% fee = 0 units
    assert_eq!(merchant_balance, 1000); // 100% = 1000 units
}

#[test]
fn test_payment_with_maximum_fee() {
    let (env, shade_client, shade_contract_id, admin, token) = setup_test_with_payment();

    // Set fee to 10000 bps (100%)
    shade_client.set_fee(&admin, &token, &10000);

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // Create merchant account
    let merchant_account = Address::generate(&env);
    shade_client.set_merchant_account(&merchant, &merchant_account);

    // Create invoice for 1000 units
    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &1000, &token);

    // Create customer and mint tokens
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&customer, &1000);

    // Customer pays invoice
    shade_client.pay_invoice(&customer, &invoice_id);

    // Verify balances
    let token_balance_client = token::TokenClient::new(&env, &token);
    let shade_balance = token_balance_client.balance(&shade_contract_id);
    let merchant_balance = token_balance_client.balance(&merchant_account);

    assert_eq!(shade_balance, 1000); // 100% fee = 1000 units
    assert_eq!(merchant_balance, 0); // 0% = 0 units
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #15)")]
fn test_payment_invoice_already_paid() {
    let (env, shade_client, _shade_contract_id, _admin, token) = setup_test_with_payment();

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // Create merchant account
    let merchant_account = Address::generate(&env);
    shade_client.set_merchant_account(&merchant, &merchant_account);

    // Create invoice
    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &1000, &token);

    // Create customer and mint tokens
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&customer, &2000);

    // Customer pays invoice first time
    shade_client.pay_invoice(&customer, &invoice_id);

    // Attempt to pay again (should panic with InvalidInvoiceStatus)
    shade_client.pay_invoice(&customer, &invoice_id);
}

#[test]
#[should_panic]
fn test_payment_insufficient_funds() {
    let (env, shade_client, _shade_contract_id, _admin, token) = setup_test_with_payment();

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // Create merchant account
    let merchant_account = Address::generate(&env);
    shade_client.set_merchant_account(&merchant, &merchant_account);

    // Create invoice for 1000 units
    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &1000, &token);

    // Create customer with insufficient balance (only 500)
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&customer, &500);

    // Customer attempts to pay invoice (should panic due to insufficient funds)
    shade_client.pay_invoice(&customer, &invoice_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #12)")]
fn test_payment_token_not_accepted() {
    let (env, shade_client, _shade_contract_id, _admin, _token) = setup_test_with_payment();

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // Create merchant account
    let merchant_account = Address::generate(&env);
    shade_client.set_merchant_account(&merchant, &merchant_account);

    // Create invoice with unaccepted token
    let unaccepted_token_admin = Address::generate(&env);
    let unaccepted_token = env.register_stellar_asset_contract_v2(unaccepted_token_admin.clone());

    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &1000, &unaccepted_token.address());

    // Create customer and mint tokens
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &unaccepted_token.address());
    token_client.mint(&customer, &1000);

    // Customer attempts to pay invoice (should panic - token not accepted)
    shade_client.pay_invoice(&customer, &invoice_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #14)")]
fn test_payment_merchant_account_not_set() {
    let (env, shade_client, _shade_contract_id, _admin, token) = setup_test_with_payment();

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // DO NOT set merchant account - this will cause the panic

    // Create invoice
    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &1000, &token);

    // Create customer and mint tokens
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&customer, &1000);

    // Customer attempts to pay invoice (should panic - merchant account not set)
    shade_client.pay_invoice(&customer, &invoice_id);
}

#[test]
fn test_payment_payer_authorization() {
    let (env, shade_client, _shade_contract_id, _admin, token) = setup_test_with_payment();

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // Create merchant account
    let merchant_account = Address::generate(&env);
    shade_client.set_merchant_account(&merchant, &merchant_account);

    // Create invoice
    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &1000, &token);

    // Create customer and mint tokens
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&customer, &1000);

    // Customer pays invoice (auth is automatically mocked)
    shade_client.pay_invoice(&customer, &invoice_id);

    // Verify payer is recorded in invoice
    let invoice = shade_client.get_invoice(&invoice_id);
    assert_eq!(invoice.payer, Some(customer));
}

#[test]
fn test_payment_updates_invoice_timestamps() {
    let (env, shade_client, _shade_contract_id, _admin, token) = setup_test_with_payment();

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // Create merchant account
    let merchant_account = Address::generate(&env);
    shade_client.set_merchant_account(&merchant, &merchant_account);

    // Create invoice
    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &1000, &token);

    // Get invoice before payment
    let invoice_before = shade_client.get_invoice(&invoice_id);
    assert!(invoice_before.date_paid.is_none());

    // Create customer and mint tokens
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&customer, &1000);

    // Customer pays invoice
    shade_client.pay_invoice(&customer, &invoice_id);

    // Get invoice after payment
    let invoice_after = shade_client.get_invoice(&invoice_id);
    assert!(invoice_after.date_paid.is_some());
    assert!(invoice_after.date_paid.unwrap() >= invoice_before.date_created);
}

#[test]
fn test_fee_calculation_accuracy() {
    let (env, shade_client, shade_contract_id, admin, token) = setup_test_with_payment();

    // Test with 1% fee (100 bps)
    shade_client.set_fee(&admin, &token, &100);

    // Register merchant
    let merchant = Address::generate(&env);
    shade_client.register_merchant(&merchant);

    // Create merchant account
    let merchant_account = Address::generate(&env);
    shade_client.set_merchant_account(&merchant, &merchant_account);

    // Create invoice for 10000 units
    let description = String::from_str(&env, "Test Invoice");
    let invoice_id = shade_client.create_invoice(&merchant, &description, &10000, &token);

    // Create customer and mint tokens
    let customer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&customer, &10000);

    // Customer pays invoice
    shade_client.pay_invoice(&customer, &invoice_id);

    // Verify balances with 1% fee
    let token_balance_client = token::TokenClient::new(&env, &token);
    let shade_balance = token_balance_client.balance(&shade_contract_id);
    let merchant_balance = token_balance_client.balance(&merchant_account);

    assert_eq!(shade_balance, 100); // 1% of 10000 = 100
    assert_eq!(merchant_balance, 9900); // 99% of 10000 = 9900
}
