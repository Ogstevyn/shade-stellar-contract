use crate::components::{access_control, admin as admin_component, merchant, signature_util};
use crate::errors::ContractError;
use crate::events;
use crate::types::{DataKey, Invoice, InvoiceFilter, InvoiceStatus, Role};
use soroban_sdk::{panic_with_error, token, Address, BytesN, Env, String, Vec};

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
    if !access_control::has_role(env, caller, Role::Manager)
        && !access_control::has_role(env, caller, Role::Admin)
    {
        panic_with_error!(env, ContractError::NotAuthorized);
    }
    caller.require_auth();

    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    if !merchant::is_merchant(env, merchant) {
        panic_with_error!(env, ContractError::MerchantNotFound);
    }

    signature_util::verify_invoice_signature(
        env,
        merchant,
        description,
        amount,
        token,
        nonce,
        signature,
    );

    signature_util::invalidate_nonce(env, merchant, nonce);

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
        merchant.clone(),
        amount,
        token.clone(),
    );

    new_invoice_id
}

pub fn pay_invoice_admin(
    env: &Env,
    caller: &Address,
    payer: &Address,
    invoice_id: u64,
) {
    // 1. Authenticate caller
    caller.require_auth();

    // 2. Authorize — must be Admin or Manager
    if !access_control::has_role(env, caller, Role::Manager)
        && !access_control::has_role(env, caller, Role::Admin)
    {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    // 3. Retrieve invoice and ensure it is Pending
    let mut invoice = get_invoice(env, invoice_id);
    if invoice.status != InvoiceStatus::Pending {
        panic_with_error!(env, ContractError::InvoiceAlreadyPaid);
    }

    // 4. Get merchant address from merchant_id
    let merchant_data: crate::types::Merchant = env
        .storage()
        .persistent()
        .get(&DataKey::Merchant(invoice.merchant_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::MerchantNotFound));

    // 5. Calculate fee
    let fee_amount = admin_component::get_fee(env, &invoice.token);
    let net_amount = invoice.amount - fee_amount;

    // 6. Transfer net amount from Shade contract to merchant address
    let token_client = token::Client::new(env, &invoice.token);
    token_client.transfer(
        &env.current_contract_address(),
        &merchant_data.address,
        &net_amount,
    );
    // fee_amount stays in the Shade contract balance as revenue

    // 7. Update invoice state
    invoice.status = InvoiceStatus::Paid;
    invoice.payer = Some(payer.clone());
    invoice.date_paid = Some(env.ledger().timestamp());
    env.storage()
        .persistent()
        .set(&DataKey::Invoice(invoice_id), &invoice);

    // 8. Emit InvoicePaid event
    events::publish_invoice_paid_event(
        env,
        invoice_id,
        payer.clone(),
        merchant_data.address,
        invoice.amount,
        fee_amount,
        invoice.token.clone(),
        env.ledger().timestamp(),
    );
}

pub fn get_invoice(env: &Env, invoice_id: u64) -> Invoice {
    env.storage()
        .persistent()
        .get(&DataKey::Invoice(invoice_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvoiceNotFound))
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
            if matches {
                invoices.push_back(invoice);
            }
        }
    }
    invoices
}