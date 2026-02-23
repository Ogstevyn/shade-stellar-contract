use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    Reentrancy = 4,
    MerchantAlreadyRegistered = 5,
    MerchantNotFound = 6,
    InvalidAmount = 7,
    InvoiceNotFound = 8,
    ContractPaused = 9,
    ContractNotPaused = 10,
    MerchantKeyNotFound = 11,
    TokenNotAccepted = 12,
    MerchantAccountNotFound = 13,
    InvalidInvoiceStatus = 14,
    RefundPeriodExpired = 15,
    WasmHashNotSet = 16,
    InvoiceAlreadyPaid = 17,
    MerchantAccountNotSet = 18,
}
