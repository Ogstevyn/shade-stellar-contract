use crate::components::{
    admin as admin_component, core as core_component, invoice as invoice_component,
    merchant as merchant_component,
};
use crate::errors::ContractError;
use crate::events;
use crate::interface::ShadeTrait;
use crate::types::{ContractInfo, DataKey, Invoice, Merchant};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, BytesN, Env, String};

#[contract]
pub struct Shade;

#[contractimpl]
impl ShadeTrait for Shade {
    fn initialize(env: Env, admin: Address) {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }
        let contract_info = ContractInfo {
            admin: admin.clone(),
            timestamp: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::ContractInfo, &contract_info);
        events::publish_initialized_event(&env, admin, env.ledger().timestamp());
    }
    fn get_admin(env: Env) -> Address {
        core_component::get_admin(&env)
    }

    fn add_accepted_token(env: Env, admin: Address, token: Address) {
        admin_component::add_accepted_token(&env, &admin, &token);
    }

    fn remove_accepted_token(env: Env, admin: Address, token: Address) {
        admin_component::remove_accepted_token(&env, &admin, &token);
    }

    fn is_accepted_token(env: Env, token: Address) -> bool {
        admin_component::is_accepted_token(&env, &token)
    }

    fn register_merchant(env: Env, merchant: Address) {
        merchant_component::register_merchant(&env, &merchant);
    }

    fn get_merchant(env: Env, merchant_id: u64) -> Merchant {
        merchant_component::get_merchant(&env, merchant_id)
    }

    fn is_merchant(env: Env, merchant: Address) -> bool {
        merchant_component::is_merchant(&env, &merchant)
    }

    fn create_invoice(
        env: Env,
        merchant: Address,
        description: String,
        amount: i128,
        token: Address,
    ) -> u64 {
        invoice_component::create_invoice(&env, &merchant, &description, amount, &token)
    }

    fn get_invoice(env: Env, invoice_id: u64) -> Invoice {
        invoice_component::get_invoice(&env, invoice_id)
    }

    fn set_merchant_key(env: Env, merchant: Address, key: BytesN<32>) {
        merchant_component::set_merchant_key(&env, &merchant, &key);
    }

    fn get_merchant_key(env: Env, merchant: Address) -> BytesN<32> {
        merchant_component::get_merchant_key(&env, &merchant)
    }
}
