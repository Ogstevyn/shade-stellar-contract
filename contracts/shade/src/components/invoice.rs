use crate::components::merchant;
use crate::errors::ContractError;
use crate::events;
use crate::types::{DataKey, Invoice, InvoiceFilter, InvoiceStatus};
use soroban_sdk::{panic_with_error, token, Address, Env, String, Vec};

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

    let date_paid = invoice.date_paid.unwrap_or(0);
    if env.ledger().timestamp() - date_paid > 604800 {
        panic_with_error!(env, ContractError::RefundWindowExpired);
    }

    let payer = invoice.payer.clone().unwrap();

    let token = token::Client::new(env, &invoice.token);
    token.transfer(&merchant_address, &payer, &amount);

    invoice.amount_refunded += amount;
    if invoice.amount_refunded == invoice.amount {
        invoice.status = InvoiceStatus::Refunded;
    } else {
        invoice.status = InvoiceStatus::PartiallyRefunded;
    }

    env.storage()
        .persistent()
        .set(&DataKey::Invoice(invoice_id), &invoice);

    events::publish_invoice_partially_refunded_event(
        env,
        invoice_id,
        merchant_address,
        amount,
        invoice.amount_refunded,
        env.ledger().timestamp(),
    );
}

pub fn refund_invoice(env: &Env, invoice_id: u64) {
    let invoice = get_invoice(env, invoice_id);
    let amount_to_refund = invoice.amount - invoice.amount_refunded;
    if amount_to_refund > 0 {
        refund_invoice_partial(env, invoice_id, amount_to_refund);
    } else {
        panic_with_error!(env, ContractError::InvalidAmount);
    }
}
