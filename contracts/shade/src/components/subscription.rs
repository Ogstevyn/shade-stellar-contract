use crate::components::{admin, merchant};
use crate::errors::ContractError;
use crate::events;
use crate::types::{DataKey, Merchant, Subscription, SubscriptionPlan, SubscriptionStatus};
use soroban_sdk::{panic_with_error, token, Address, Env, String};

pub fn create_plan(
    env: &Env,
    merchant_address: &Address,
    description: &String,
    amount: i128,
    token: &Address,
    interval: u64,
) -> u64 {
    merchant_address.require_auth();

    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }
    if interval == 0 {
        panic_with_error!(env, ContractError::InvalidInterval);
    }
    if !merchant::is_merchant(env, merchant_address) {
        panic_with_error!(env, ContractError::MerchantNotFound);
    }

    let merchant_id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantId(merchant_address.clone()))
        .unwrap();

    let plan_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::PlanCount)
        .unwrap_or(0);
    let new_plan_id = plan_count + 1;

    let plan = SubscriptionPlan {
        id: new_plan_id,
        merchant_id,
        description: description.clone(),
        amount,
        token: token.clone(),
        interval,
        active: true,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Plan(new_plan_id), &plan);
    env.storage()
        .persistent()
        .set(&DataKey::PlanCount, &new_plan_id);

    events::publish_plan_created_event(
        env,
        new_plan_id,
        merchant_address.clone(),
        amount,
        interval,
        env.ledger().timestamp(),
    );

    new_plan_id
}

pub fn subscribe(env: &Env, customer: &Address, plan_id: u64) -> u64 {
    customer.require_auth();

    let plan: SubscriptionPlan = env
        .storage()
        .persistent()
        .get(&DataKey::Plan(plan_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::PlanNotFound));

    if !plan.active {
        panic_with_error!(env, ContractError::PlanNotActive);
    }

    let sub_count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::SubscriptionCount)
        .unwrap_or(0);
    let new_sub_id = sub_count + 1;

    let subscription = Subscription {
        id: new_sub_id,
        plan_id,
        customer: customer.clone(),
        merchant_id: plan.merchant_id,
        status: SubscriptionStatus::Active,
        date_created: env.ledger().timestamp(),
        last_charged: 0,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Subscription(new_sub_id), &subscription);
    env.storage()
        .persistent()
        .set(&DataKey::SubscriptionCount, &new_sub_id);

    events::publish_subscription_created_event(
        env,
        new_sub_id,
        plan_id,
        customer.clone(),
        env.ledger().timestamp(),
    );

    new_sub_id
}

pub fn charge_subscription(env: &Env, subscription_id: u64) {
    let mut subscription: Subscription = env
        .storage()
        .persistent()
        .get(&DataKey::Subscription(subscription_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::SubscriptionNotFound));

    if subscription.status != SubscriptionStatus::Active {
        panic_with_error!(env, ContractError::SubscriptionNotActive);
    }

    let plan: SubscriptionPlan = env
        .storage()
        .persistent()
        .get(&DataKey::Plan(subscription.plan_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::PlanNotFound));

    // Enforce interval: first charge is immediate (last_charged == 0),
    // subsequent charges must wait for the full interval to elapse.
    let now = env.ledger().timestamp();
    if subscription.last_charged > 0 {
        let next_charge_time = subscription.last_charged + plan.interval;
        if now < next_charge_time {
            panic_with_error!(env, ContractError::ChargeTooEarly);
        }
    }

    // Calculate fee split
    let fee_amount = get_fee_for_plan(env, &plan);
    let merchant_amount = plan.amount - fee_amount;

    let token_client = token::TokenClient::new(env, &plan.token);
    let merchant_account_id = merchant::get_merchant_account(env, plan.merchant_id);
    let shade_contract = env.current_contract_address();

    // Use transfer_from: shade contract spends the customer's approved allowance
    token_client.transfer_from(
        &shade_contract,
        &subscription.customer,
        &merchant_account_id,
        &merchant_amount,
    );

    if fee_amount > 0 {
        token_client.transfer_from(
            &shade_contract,
            &subscription.customer,
            &shade_contract,
            &fee_amount,
        );
    }

    subscription.last_charged = now;
    env.storage()
        .persistent()
        .set(&DataKey::Subscription(subscription_id), &subscription);

    events::publish_subscription_charged_event(
        env,
        subscription_id,
        plan.amount,
        fee_amount,
        env.ledger().timestamp(),
    );
}

pub fn cancel_subscription(env: &Env, caller: &Address, subscription_id: u64) {
    caller.require_auth();

    let mut subscription: Subscription = env
        .storage()
        .persistent()
        .get(&DataKey::Subscription(subscription_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::SubscriptionNotFound));

    if subscription.status != SubscriptionStatus::Active {
        panic_with_error!(env, ContractError::SubscriptionNotActive);
    }

    // Cancellation is allowed by the customer or the plan's merchant
    let is_customer = *caller == subscription.customer;
    let is_merchant = {
        let plan: SubscriptionPlan = env
            .storage()
            .persistent()
            .get(&DataKey::Plan(subscription.plan_id))
            .unwrap();
        let merchant_record: Merchant = env
            .storage()
            .persistent()
            .get(&DataKey::Merchant(plan.merchant_id))
            .unwrap();
        *caller == merchant_record.address
    };

    if !is_customer && !is_merchant {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    subscription.status = SubscriptionStatus::Cancelled;
    env.storage()
        .persistent()
        .set(&DataKey::Subscription(subscription_id), &subscription);

    events::publish_subscription_cancelled_event(
        env,
        subscription_id,
        caller.clone(),
        env.ledger().timestamp(),
    );
}

pub fn get_plan(env: &Env, plan_id: u64) -> SubscriptionPlan {
    env.storage()
        .persistent()
        .get(&DataKey::Plan(plan_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::PlanNotFound))
}

pub fn get_subscription(env: &Env, subscription_id: u64) -> Subscription {
    env.storage()
        .persistent()
        .get(&DataKey::Subscription(subscription_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::SubscriptionNotFound))
}

fn get_fee_for_plan(env: &Env, plan: &SubscriptionPlan) -> i128 {
    let fee: i128 = admin::get_fee(env, &plan.token);
    if fee == 0 {
        return 0;
    }
    (plan.amount * fee) / 10_000i128
}
