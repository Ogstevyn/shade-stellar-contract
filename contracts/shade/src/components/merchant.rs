use crate::errors::ContractError;
use crate::events;
use crate::types::{DataKey, Merchant, MerchantFilter};
use soroban_sdk::{panic_with_error, Address, Env, Vec};

pub fn register_merchant(env: &Env, merchant: &Address) {
    merchant.require_auth();

    if env
        .storage()
        .persistent()
        .has(&DataKey::MerchantId(merchant.clone()))
    {
        panic_with_error!(env, ContractError::MerchantAlreadyRegistered);
    }

    let merchant_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantCount)
        .unwrap_or(0);

    let new_id = merchant_count + 1;

    let merchant_data = Merchant {
        id: new_id,
        address: merchant.clone(),
        active: true,
        verified: false,
        date_registered: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DataKey::Merchant(new_id), &merchant_data);
    env.storage()
        .persistent()
        .set(&DataKey::MerchantId(merchant.clone()), &new_id);
    env.storage()
        .persistent()
        .set(&DataKey::MerchantCount, &new_id);

    events::publish_merchant_registered_event(
        env,
        merchant.clone(),
        new_id,
        env.ledger().timestamp(),
    );
}

pub fn get_merchant(env: &Env, merchant_id: u64) -> Merchant {
    if merchant_id == 0 {
        panic_with_error!(env, ContractError::MerchantNotFound);
    }

    let merchant_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantCount)
        .unwrap_or(0);

    if merchant_id > merchant_count {
        panic_with_error!(env, ContractError::MerchantNotFound);
    }

    env.storage()
        .persistent()
        .get(&DataKey::Merchant(merchant_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::MerchantNotFound))
}

pub fn is_merchant(env: &Env, merchant: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::MerchantId(merchant.clone()))
}

pub fn get_merchants(env: &Env, filter: MerchantFilter) -> Vec<Merchant> {
    let merchant_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantCount)
        .unwrap_or(0);

    let mut merchants: Vec<Merchant> = Vec::new(env);

    for i in 1..=merchant_count {
        if let Some(merchant) = env
            .storage()
            .persistent()
            .get::<_, Merchant>(&DataKey::Merchant(i))
        {
            let mut matches = true;

            if let Some(active) = filter.is_active {
                if merchant.active != active {
                    matches = false;
                }
            }

            if let Some(verified) = filter.is_verified {
                if merchant.verified != verified {
                    matches = false;
                }
            }

            if matches {
                merchants.push_back(merchant);
            }
        }
    }

    merchants
}
