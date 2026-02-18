use crate::errors::ContractError;
use crate::events::publish_account_initialized_event;
use crate::interface::MerchantAccountTrait;
use crate::types::{AccountInfo, DataKey};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env};

#[contract]
pub struct MerchantAccount;

#[contractimpl]
impl MerchantAccountTrait for MerchantAccount {
    fn initialize(env: Env, merchant: Address, manager: Address, merchant_id: u64) {
        if env.storage().persistent().has(&DataKey::Merchant) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }
        let account_info = AccountInfo {
            merchant: merchant.clone(),
            manager: manager.clone(),
            merchant_id,
            date_created: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::AccountInfo, &account_info);
        env.storage()
            .persistent()
            .set(&DataKey::Merchant, &merchant);
        env.storage().persistent().set(&DataKey::Manager, &manager);
        publish_account_initialized_event(
            &env,
            merchant.clone(),
            merchant_id,
            env.ledger().timestamp(),
        );
    }
    fn get_merchant(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Merchant)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NotInitialized))
    }
}
