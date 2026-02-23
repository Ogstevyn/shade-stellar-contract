use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Admin,
    Paused,
    FeeInBasisPoints(Address),
    FeeAmount(Address),
    ContractInfo,
    AcceptedTokens,
    Merchant(u64),
    MerchantKey(Address),
    MerchantCount,
    MerchantId(Address),
    TokenFee(Address),
    MerchantTokens,
    MerchantBalance(Address),
    Invoice(u64),
    InvoiceCount,
    ReentrancyStatus,
    Role(Address, Role),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractInfo {
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merchant {
    pub id: u64,
    pub address: Address,
    pub active: bool,
    pub verified: bool,
    pub date_registered: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invoice {
    pub id: u64,
    pub description: soroban_sdk::String,
    pub amount: i128,
    pub token: Address,
    pub status: InvoiceStatus,
    pub merchant_id: u64,
    pub payer: Option<Address>,
    pub date_created: u64,
    pub date_paid: Option<u64>,
    pub amount_refunded: i128,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum InvoiceStatus {
    Pending = 0,
    Paid = 1,
    Cancelled = 2,
    Refunded = 3,
    PartiallyRefunded = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantFilter {
    pub is_active: Option<bool>,
    pub is_verified: Option<bool>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceFilter {
    pub status: Option<u32>,
    pub merchant: Option<Address>,
    pub min_amount: Option<u128>,
    pub max_amount: Option<u128>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Manager,
    Operator,
}
