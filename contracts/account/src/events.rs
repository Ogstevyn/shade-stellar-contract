use soroban_sdk::{contractevent, Address, Env};

#[contractevent]
pub struct AccountInitalizedEvent {
    pub merchant: Address,
    pub merchant_id: u64,
    pub timestamp: u64,
}

pub fn publish_account_initialized_event(
    env: &Env,
    merchant: Address,
    merchant_id: u64,
    timestamp: u64,
) {
    AccountInitalizedEvent {
        merchant,
        merchant_id,
        timestamp,
    }
    .publish(env);
}
