use soroban_sdk::{contractevent, Address, BytesN, Env};

#[contractevent]
pub struct InitalizedEvent {
    pub admin: Address,
    pub timestamp: u64,
}

pub fn publish_initialized_event(env: &Env, admin: Address, timestamp: u64) {
    InitalizedEvent { admin, timestamp }.publish(env);
}

#[contractevent]
pub struct TokenAddedEvent {
    pub token: Address,
    pub timestamp: u64,
}

pub fn publish_token_added_event(env: &Env, token: Address, timestamp: u64) {
    TokenAddedEvent { token, timestamp }.publish(env);
}

#[contractevent]
pub struct TokenRemovedEvent {
    pub token: Address,
    pub timestamp: u64,
}

pub fn publish_token_removed_event(env: &Env, token: Address, timestamp: u64) {
    TokenRemovedEvent { token, timestamp }.publish(env);
}

#[contractevent]
pub struct MerchantRegisteredEvent {
    pub merchant: Address,
    pub merchant_id: u64,
    pub timestamp: u64,
}

pub fn publish_merchant_registered_event(
    env: &Env,
    merchant: Address,
    merchant_id: u64,
    timestamp: u64,
) {
    MerchantRegisteredEvent {
        merchant,
        merchant_id,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct MerchantAccountDeployedEvent {
    pub merchant: Address,
    pub contract: Address,
    pub timestamp: u64,
}

pub fn publish_merchant_account_deployed_event(
    env: &Env,
    merchant: Address,
    contract: Address,
    timestamp: u64,
) {
    MerchantAccountDeployedEvent {
        merchant,
        contract,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct MerchantStatusChangedEvent {
    pub merchant_id: u64,
    pub active: bool,
    pub timestamp: u64,
}

pub fn publish_merchant_status_changed_event(
    env: &Env,
    merchant_id: u64,
    active: bool,
    timestamp: u64,
) {
    MerchantStatusChangedEvent {
        merchant_id,
        active,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct InvoiceCreatedEvent {
    pub invoice_id: u64,
    pub merchant: Address,
    pub amount: i128,
    pub token: Address,
}

pub fn publish_invoice_created_event(
    env: &Env,
    invoice_id: u64,
    merchant: Address,
    amount: i128,
    token: Address,
) {
    InvoiceCreatedEvent {
        invoice_id,
        merchant,
        amount,
        token,
    }
    .publish(env);
}

#[contractevent]
pub struct InvoiceRefundedEvent {
    pub invoice_id: u64,
    pub merchant: Address,
    pub amount: i128,
    pub timestamp: u64,
}

pub fn publish_invoice_refunded_event(
    env: &Env,
    invoice_id: u64,
    merchant: Address,
    amount: i128,
    timestamp: u64,
) {
    InvoiceRefundedEvent {
        invoice_id,
        merchant,
        amount,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct InvoicePartiallyRefundedEvent {
    pub invoice_id: u64,
    pub merchant: Address,
    pub amount: i128,
    pub total_amount_refunded: i128,
    pub timestamp: u64,
}

pub fn publish_invoice_partially_refunded_event(
    env: &Env,
    invoice_id: u64,
    merchant: Address,
    amount: i128,
    total_amount_refunded: i128,
    timestamp: u64,
) {
    InvoicePartiallyRefundedEvent {
        invoice_id,
        merchant,
        amount,
        total_amount_refunded,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct MerchantVerifiedEvent {
    pub merchant_id: u64,
    pub status: bool,
    pub timestamp: u64,
}

pub fn publish_merchant_verified_event(env: &Env, merchant_id: u64, status: bool, timestamp: u64) {
    MerchantVerifiedEvent {
        merchant_id,
        status,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct MerchantKeySetEvent {
    pub merchant: Address,
    pub key: BytesN<32>,
    pub timestamp: u64,
}

pub fn publish_merchant_key_set_event(
    env: &Env,
    merchant: Address,
    key: BytesN<32>,
    timestamp: u64,
) {
    MerchantKeySetEvent {
        merchant,
        key,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct RoleGrantedEvent {
    pub user: Address,
    pub role: crate::types::Role,
    pub timestamp: u64,
}

pub fn publish_role_granted_event(
    env: &Env,
    user: Address,
    role: crate::types::Role,
    timestamp: u64,
) {
    RoleGrantedEvent {
        user,
        role,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct RoleRevokedEvent {
    pub user: Address,
    pub role: crate::types::Role,
    pub timestamp: u64,
}

pub fn publish_role_revoked_event(
    env: &Env,
    user: Address,
    role: crate::types::Role,
    timestamp: u64,
) {
    RoleRevokedEvent {
        user,
        role,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct ContractPausedEvent {
    pub admin: Address,
    pub timestamp: u64,
}

pub fn publish_contract_paused_event(env: &Env, admin: Address, timestamp: u64) {
    ContractPausedEvent { admin, timestamp }.publish(env);
}

#[contractevent]
pub struct ContractUnpausedEvent {
    pub admin: Address,
    pub timestamp: u64,
}

pub fn publish_contract_unpaused_event(env: &Env, admin: Address, timestamp: u64) {
    ContractUnpausedEvent { admin, timestamp }.publish(env);
}

#[contractevent]
pub struct FeeSetEvent {
    pub token: Address,
    pub fee: i128,
    pub timestamp: u64,
}

pub fn publish_fee_set_event(env: &Env, token: Address, fee: i128, timestamp: u64) {
    FeeSetEvent {
        token,
        fee,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct ContractUpgradedEvent {
    pub new_wasm_hash: BytesN<32>,
    pub timestamp: u64,
}

pub fn publish_contract_upgraded_event(env: &Env, new_wasm_hash: BytesN<32>, timestamp: u64) {
    ContractUpgradedEvent {
        new_wasm_hash,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct InvoicePaidEvent {
    pub invoice_id: u64,
    pub payer: Address,
    pub amount: i128,
    pub fee: i128,
    pub merchant_amount: i128,
    pub timestamp: u64,
}

pub fn publish_invoice_paid_event(
    env: &Env,
    invoice_id: u64,
    payer: Address,
    amount: i128,
    fee: i128,
    merchant_amount: i128,
    timestamp: u64,
) {
    InvoicePaidEvent {
        invoice_id,
        payer,
        amount,
        fee,
        merchant_amount,
        timestamp,
    }
    .publish(env);
}

#[contractevent]
pub struct InvoiceCancelledEvent {
    pub invoice_id: u64,
    pub merchant: Address,
    pub timestamp: u64,
}

pub fn publish_invoice_cancelled_event(
    env: &Env,
    invoice_id: u64,
    merchant: Address,
    timestamp: u64,
) {
    InvoiceCancelledEvent {
        invoice_id,
        merchant,
        timestamp,
    }
    .publish(env);
}
