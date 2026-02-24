use crate::components::{admin, merchant};
use crate::errors::ContractError;
use crate::events;
use crate::types::{DataKey, Invoice, InvoiceFilter, InvoiceStatus};
use account::account::MerchantAccountClient;
use soroban_sdk::{panic_with_error, token, Address, Env, String, Vec};

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

            if matches {
                invoices.push_back(invoice);
            }
        }
    }

    invoices
}

pub fn refund_invoice_partial(env: &Env, invoice_id: u64, amount: i128) {
    let mut invoice = get_invoice(env, invoice_id);

    let merchant_address = merchant::get_merchant(env, invoice.merchant_id).address;
    merchant_address.require_auth();

    if invoice.status != InvoiceStatus::Paid && invoice.status != InvoiceStatus::PartiallyRefunded {
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    if amount <= 0 || invoice.amount_refunded + amount > invoice.amount {
        panic_with_error!(env, ContractError::InvalidAmount);
    }

    let date_paid = invoice
        .date_paid
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidInvoiceStatus));
    let now = env.ledger().timestamp();
    if now < date_paid || now - date_paid > MAX_REFUND_DURATION {
        panic_with_error!(env, ContractError::RefundPeriodExpired);
    }

    let payer = invoice
        .payer
        .clone()
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidInvoiceStatus));
    let merchant_account: Address = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantBalance(merchant_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::MerchantAccountNotFound));
    let token = invoice.token.clone();
    MerchantAccountClient::new(env, &merchant_account).refund(&token, &amount, &payer);

    invoice.amount_refunded += amount;
    let is_fully_refunded = invoice.amount_refunded == invoice.amount;
    invoice.status = if is_fully_refunded {
        InvoiceStatus::Refunded
    } else {
        InvoiceStatus::PartiallyRefunded
    };

    env.storage()
        .persistent()
        .set(&DataKey::Invoice(invoice_id), &invoice);

    if is_fully_refunded {
        events::publish_invoice_refunded_event(env, invoice_id, merchant_address, amount, now);
    } else {
        events::publish_invoice_partially_refunded_event(
            env,
            invoice_id,
            merchant_address,
            amount,
            invoice.amount_refunded,
            now,
        );
    }
}

pub fn pay_invoice(env: &Env, payer: &Address, invoice_id: u64) {
    payer.require_auth();

    // Get invoice
    let mut invoice = get_invoice(env, invoice_id);

    // Check invoice status
    if invoice.status != InvoiceStatus::Pending {
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    // Check token is accepted
    if !admin::is_accepted_token(env, &invoice.token) {
        panic_with_error!(env, ContractError::TokenNotAccepted);
    }

    // Get fee in basis points (e.g., 500 = 5%)
    let fee_bps = admin::get_fee(env, &invoice.token);

    // Calculate fee and merchant amount
    // fee = (amount * fee_bps) / 10000
    let fee_amount = (invoice.amount * fee_bps) / 10000;
    let merchant_amount = invoice.amount - fee_amount;

    // Get merchant account address
    let merchant_account = merchant::get_merchant_account(env, invoice.merchant_id);

    // Get token client
    let token_client = token::TokenClient::new(env, &invoice.token);
    let shade_contract = env.current_contract_address();

    // Transfer fee to Shade contract
    if fee_amount > 0 {
        token_client.transfer(payer, &shade_contract, &fee_amount);
    }

    // Transfer merchant amount to merchant account
    if merchant_amount > 0 {
        token_client.transfer(payer, &merchant_account, &merchant_amount);
    }

    // Update invoice
    invoice.status = InvoiceStatus::Paid;
    invoice.payer = Some(payer.clone());
    invoice.date_paid = Some(env.ledger().timestamp());

    env.storage()
        .persistent()
        .set(&DataKey::Invoice(invoice_id), &invoice);

    // Emit event
    events::publish_invoice_paid_event(
        env,
        invoice_id,
        payer.clone(),
        invoice.amount,
        fee_amount,
        merchant_amount,
        env.ledger().timestamp(),
    );
}

pub fn void_invoice(env: &Env, merchant_address: &Address, invoice_id: u64) {
    merchant_address.require_auth();

    // Get invoice
    let mut invoice = get_invoice(env, invoice_id);

    // Get merchant ID for ownership check
    let merchant_id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantId(merchant_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::NotAuthorized));

    // Verify merchant owns this invoice
    if invoice.merchant_id != merchant_id {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    // Verify invoice status is Pending
    if invoice.status != InvoiceStatus::Pending {
        panic_with_error!(env, ContractError::InvalidInvoiceStatus);
    }

    // Update status to Cancelled
    invoice.status = InvoiceStatus::Cancelled;

    // Save updated invoice
    env.storage()
        .persistent()
        .set(&DataKey::Invoice(invoice_id), &invoice);

    // Emit event
    events::publish_invoice_cancelled_event(
        env,
        invoice_id,
        merchant_address.clone(),
        env.ledger().timestamp(),
    );
}
