use crate::components::{access_control, admin, merchant, signature_util};
use crate::errors::ContractError;
use crate::events;
use crate::types::{DataKey, Invoice, InvoiceFilter, InvoiceStatus, Role};
use account::account::MerchantAccountClient;
use soroban_sdk::{panic_with_error, token, Address, BytesN, Env, String, Vec};

pub const MAX_REFUND_DURATION: u64 = 604_800;

pub fn create_invoice(
    env: &Env,
    merchant_address: &Address,
    description: &String,
    amount: i128,
    token: &Address,
) -> u64 {
    merchant_address.require_auth();
    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }
    if !merchant::is_merchant(env, merchant_address) {
        panic_with_error!(env, ContractError::NotAuthorized);
    }
    let merchant_id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantId(merchant_address.clone()))
        .unwrap();
    let invoice_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::InvoiceCount)
        .unwrap_or(0);
    let new_invoice_id = invoice_count + 1;
    let invoice = Invoice {
        id: new_invoice_id,
        description: description.clone(),
        amount,
        token: token.clone(),
        status: InvoiceStatus::Pending,
        merchant_id,
        payer: None,
        date_created: env.ledger().timestamp(),
        date_paid: None,
        amount_refunded: 0,
    };
    env.storage()
        .persistent()
        .set(&DataKey::Invoice(new_invoice_id), &invoice);
    env.storage()
        .persistent()
        .set(&DataKey::InvoiceCount, &new_invoice_id);
    events::publish_invoice_created_event(
        env,
        new_invoice_id,
        merchant_address.clone(),
        amount,
        token.clone(),
    );
    new_invoice_id
}

pub fn create_invoice_signed(
    env: &Env,
    caller: &Address,
    merchant: &Address,
    description: &String,
    amount: i128,
    token: &Address,
    nonce: &BytesN<32>,
    signature: &BytesN<64>,
) -> u64 {
    // 1. Caller must be Manager or Admin
    if !access_control::has_role(env, caller, Role::Manager)
        && !access_control::has_role(env, caller, Role::Admin)
    {
        panic_with_error!(env, ContractError::NotAuthorized);
    }
    caller.require_auth();

    // 2. Validate amount
    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    // 3. Merchant must exist
    if !merchant::is_merchant(env, merchant) {
        panic_with_error!(env, ContractError::MerchantNotFound);
    }

    // 4. Verify merchant's cryptographic signature
    signature_util::verify_invoice_signature(
        env,
        merchant,
        description,
        amount,
        token,
        nonce,
        signature,
    );

    // 5. Invalidate nonce to prevent replay attacks
    signature_util::invalidate_nonce(env, merchant, nonce);

    // 6. Standard invoice creation
    let merchant_id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantId(merchant.clone()))
        .unwrap();

    let invoice_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::InvoiceCount)
        .unwrap_or(0);

    let new_invoice_id = invoice_count + 1;

    let invoice = Invoice {
        id: new_invoice_id,
        description: description.clone(),
        amount,
        token: token.clone(),
        status: InvoiceStatus::Pending,
        merchant_id,
        payer: None,
        date_created: env.ledger().timestamp(),
        date_paid: None,
        amount_refunded: 0,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Invoice(new_invoice_id), &invoice);
    env.storage()
        .persistent()
        .set(&DataKey::InvoiceCount, &new_invoice_id);

    // 7. Emit standard InvoiceCreated event
    events::publish_invoice_created_event(
        env,
        new_invoice_id,
        merchant.clone(),
        amount,
        token.clone(),
    );

    new_invoice_id
}

pub fn get_invoice(env: &Env, invoice_id: u64) -> Invoice {
    env.storage()
        .persistent()
        .get(&DataKey::Invoice(invoice_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound))
}

pub fn refund_invoice(env: &Env, merchant_address: &Address, invoice_id: u64) {
    merchant_address.require_auth();

    let invoice = get_invoice(env, invoice_id);

    let merchant_id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantId(merchant_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::NotAuthorized));

    if invoice.merchant_id != merchant_id {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    let amount_to_refund = invoice.amount - invoice.amount_refunded;
    if amount_to_refund <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    refund_invoice_partial(env, invoice_id, amount_to_refund);
}

pub fn get_invoices(env: &Env, filter: InvoiceFilter) -> Vec<Invoice> {
    let invoice_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::InvoiceCount)
        .unwrap_or(0);
    let mut invoices: Vec<Invoice> = Vec::new(env);
    for i in 1..=invoice_count {
        if let Some(invoice) = env
            .storage()
            .persistent()
            .get::<_, Invoice>(&DataKey::Invoice(i))
        {
            let mut matches = true;
            if let Some(status) = filter.status {
                if invoice.status as u32 != status {
                    matches = false;
                }
            }
            if let Some(merchant) = &filter.merchant {
                if let Some(merchant_id) = env
                    .storage()
                    .persistent()
                    .get::<_, u64>(&DataKey::MerchantId(merchant.clone()))
                {
                    if invoice.merchant_id != merchant_id {
                        matches = false;
                    }
                } else {
                    matches = false;
                }
            }
            if let Some(min_amount) = filter.min_amount {
                if invoice.amount < min_amount as i128 {
                    matches = false;
                }
            }
            if let Some(max_amount) = filter.max_amount {
                if invoice.amount > max_amount as i128 {
                    matches = false;
                }
            }
            if let Some(start_date) = filter.start_date {
                if invoice.date_created < start_date {
                    matches = false;
                }
            }
            if let Some(end_date) = filter.end_date {
    