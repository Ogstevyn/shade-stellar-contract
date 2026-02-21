use crate::types::{Invoice, Merchant};
use soroban_sdk::{contracttrait, Address, BytesN, Env, String};

#[contracttrait]
pub trait ShadeTrait {
    fn initialize(env: Env, admin: Address);
    fn get_admin(env: Env) -> Address;
    fn add_accepted_token(env: Env, admin: Address, token: Address);
    fn remove_accepted_token(env: Env, admin: Address, token: Address);
    fn is_accepted_token(env: Env, token: Address) -> bool;
    fn register_merchant(env: Env, merchant: Address);
    fn get_merchant(env: Env, merchant_id: u64) -> Merchant;
    fn is_merchant(env: Env, merchant: Address) -> bool;
    fn create_invoice(
        env: Env,
        merchant: Address,
        description: String,
        amount: i128,
        token: Address,
    ) -> u64;
    fn get_invoice(env: Env, invoice_id: u64) -> Invoice;
    fn set_merchant_key(env: Env, merchant: Address, key: BytesN<32>);
    fn get_merchant_key(env: Env, merchant: Address) -> BytesN<32>;
}
