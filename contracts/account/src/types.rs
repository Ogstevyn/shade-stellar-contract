use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Manager,
    Merchant,
    Verified,
    AccountInfo,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountInfo {
    pub manager: Address,
    pub merchant_id: u64,
    pub merchant: Address,
    pub date_created: u64,
}
