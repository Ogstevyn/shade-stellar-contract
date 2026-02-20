use crate::types::{Invoice, Merchant, Role};
use soroban_sdk::{contracttrait, Address, Env, String};

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
    fn grant_role(env: Env, admin: Address, user: Address, role: Role);
    fn revoke_role(env: Env, admin: Address, user: Address, role: Role);
    fn has_role(env: Env, user: Address, role: Role) -> bool;
}
