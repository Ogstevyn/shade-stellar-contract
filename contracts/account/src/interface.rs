use soroban_sdk::{contracttrait, Address, Env};

#[contracttrait]
pub trait MerchantAccountTrait {
    fn initialize(env: Env, merchant: Address, manager: Address, merchant_id: u64);
    fn get_merchant(env: Env) -> Address;
}
