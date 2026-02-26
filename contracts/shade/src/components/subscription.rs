// ============================================================================
//  contracts/shade/src/components/subscription.rs
//
//  Drop-in subscription engine for the Shade contract.
//  These four functions are called from within the `#[contractimpl]` block
//  in shade.rs — see the integration comment at the bottom of this file.
// ============================================================================

use crate::errors::ContractError;
use crate::events::{
    publish_subscribed_event, publish_subscription_cancelled_event,
    publish_subscription_charged_event, publish_subscription_plan_created_event,
};
use crate::types::{DataKey, Subscription, SubscriptionPlan, SubscriptionStatus};
use soroban_sdk::{panic_with_error, token, Address, Env};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Read the current plan counter (defaults to 0 if never set).
fn get_plan_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::PlanCount)
        .unwrap_or(0u64)
}

/// Read the current subscription counter (defaults to 0 if never set).
fn get_subscription_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::SubscriptionCount)
        .unwrap_or(0u64)
}

/// Load a plan by ID, panic if it does not exist.
fn load_plan(env: &Env, plan_id: u64) -> SubscriptionPlan {
    env.storage()
        .persistent()
        .get(&DataKey::SubscriptionPlan(plan_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::NotFound))
}

/// Load a subscription by ID, panic if it does not exist.
fn load_subscription(env: &Env, subscription_id: u64) -> Subscription {
    env.storage()
        .persistent()
        .get(&DataKey::Subscription(subscription_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::NotFound))
}

// ── public functions (called from ShadeTrait impl) ────────────────────────────

/// Create a recurring billing plan.
///
/// * `merchant`  – address that owns the plan; must authenticate.
/// * `token`     – token used for billing.
/// * `amount`    – amount per billing cycle (in token base units, must be > 0).
/// * `interval`  – seconds between charges (must be > 0).
///
/// Returns the newly assigned plan ID.
pub fn create_subscription_plan(
    env: &Env,
    merchant: Address,
    token: Address,
    amount: i128,
    interval: u64,
) -> u64 {
    // Only the merchant can create their own plan.
    merchant.require_auth();

    if amount <= 0 {
        panic_with_error!(env, ContractError::InvalidAmount);
    }
    if interval == 0 {
        panic_with_error!(env, ContractError::InvalidInterval);
    }

    let plan_id = get_plan_count(env) + 1;

    let plan = SubscriptionPlan {
        id: plan_id,
        merchant: merchant.clone(),
        token: token.clone(),
        amount,
        interval,
        active: true,
    };

    env.storage()
        .persistent()
        .set(&DataKey::SubscriptionPlan(plan_id), &plan);
    env.storage()
        .persistent()
        .set(&DataKey::PlanCount, &plan_id);

    publish_subscription_plan_created_event(
        env,
        plan_id,
        merchant,
        token,
        amount,
        interval,
        env.ledger().timestamp(),
    );

    plan_id
}

/// Fetch a plan by ID.
pub fn get_subscription_plan(env: &Env, plan_id: u64) -> SubscriptionPlan {
    load_plan(env, plan_id)
}

/// Subscribe a customer to an active plan.
///
/// The customer must have called `token.approve(shade_contract, amount)`
/// (or a higher allowance) before recurring charges will succeed.
///
/// Returns the newly assigned subscription ID.
pub fn subscribe(env: &Env, customer: Address, plan_id: u64) -> u64 {
    // Customer must authenticate.
    customer.require_auth();

    let plan = load_plan(env, plan_id);

    if !plan.active {
        panic_with_error!(env, ContractError::PlanNotActive);
    }

    let subscription_id = get_subscription_count(env) + 1;

    let subscription = Subscription {
        id: subscription_id,
        plan_id,
        customer: customer.clone(),
        // last_charge_date = 0 means the first charge is available immediately.
        last_charge_date: 0,
        status: SubscriptionStatus::Active,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Subscription(subscription_id), &subscription);
    env.storage()
        .persistent()
        .set(&DataKey::SubscriptionCount, &subscription_id);

    publish_subscribed_event(
        env,
        subscription_id,
        plan_id,
        customer,
        env.ledger().timestamp(),
    );

    subscription_id
}

/// Fetch a subscription by ID.
pub fn get_subscription(env: &Env, subscription_id: u64) -> Subscription {
    load_subscription(env, subscription_id)
}

/// Charge a subscription.
///
/// Anyone may call this (typically the merchant or an automated bot).
/// The function:
///   1. Verifies the subscription is Active.
///   2. Checks that `now >= last_charge_date + interval`.
///   3. Reads the protocol fee for the token.
///   4. Uses `transfer_from` to pull `amount` from the customer's wallet:
///      - `fee` portion goes to the Shade contract itself.
///      - `net` portion goes to the merchant's account contract.
///   5. Updates `last_charge_date` and persists the subscription.
///   6. Emits `SubscriptionChargedEvent`.
pub fn charge_subscription(env: &Env, subscription_id: u64) {
    let mut subscription = load_subscription(env, subscription_id);

    // Must still be active.
    if subscription.status != SubscriptionStatus::Active {
        panic_with_error!(env, ContractError::SubscriptionNotActive);
    }

    let plan = load_plan(env, subscription.plan_id);
    let now = env.ledger().timestamp();

    // Enforce the billing interval.
    if now < subscription.last_charge_date + plan.interval {
        panic_with_error!(env, ContractError::IntervalNotElapsed);
    }

    // ── Fee calculation (mirrors existing invoice fee logic) ──────────────────
    // Fee is stored in basis points (1 bp = 0.01 %).
    // If no fee is configured for this token, fee = 0.
    let fee_bps: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::FeeInBasisPoints(plan.token.clone()))
        .unwrap_or(0i128);

    let fee: i128 = (plan.amount * fee_bps) / 10_000;
    let net: i128 = plan.amount - fee;

    // ── Token transfer (pull model via transfer_from) ─────────────────────────
    let token_client = token::TokenClient::new(env, &plan.token);
    let shade_contract = env.current_contract_address();

    // Retrieve the merchant's deployed account contract address.
    // We use MerchantId(address) → u64 to look up the merchant ID, then
    // MerchantAccount(id) → Address to get the account contract.
    let merchant_id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantId(plan.merchant.clone()))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::MerchantNotFound));

    let merchant_account: Address = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantAccount(merchant_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::MerchantAccountNotFound));

    // Pull fee to Shade contract.
    if fee > 0 {
        token_client.transfer_from(
            &shade_contract,
            &subscription.customer,
            &shade_contract,
            &fee,
        );
    }

    // Pull net amount to merchant account.
    token_client.transfer_from(
        &shade_contract,
        &subscription.customer,
        &merchant_account,
        &net,
    );

    // ── Update state ──────────────────────────────────────────────────────────
    subscription.last_charge_date = now;
    env.storage()
        .persistent()
        .set(&DataKey::Subscription(subscription_id), &subscription);

    publish_subscription_charged_event(
        env,
        subscription_id,
        plan.id,
        subscription.customer,
        plan.merchant,
        plan.amount,
        fee,
        plan.token,
        now,
    );
}

/// Cancel a subscription.
///
/// Either the **customer** or the **merchant** may cancel.
/// Panics if the caller is neither.
pub fn cancel_subscription(env: &Env, caller: Address, subscription_id: u64) {
    caller.require_auth();

    let mut subscription = load_subscription(env, subscription_id);
    let plan = load_plan(env, subscription.plan_id);

    // Only the customer or the merchant that owns the plan may cancel.
    if caller != subscription.customer && caller != plan.merchant {
        panic_with_error!(env, ContractError::NotAuthorized);
    }

    subscription.status = SubscriptionStatus::Cancelled;
    env.storage()
        .persistent()
        .set(&DataKey::Subscription(subscription_id), &subscription);

    publish_subscription_cancelled_event(
        env,
        subscription_id,
        caller,
        env.ledger().timestamp(),
    );
}

// ============================================================================
//  HOW TO WIRE THIS INTO shade.rs
// ============================================================================
//
//  1. At the top of shade.rs add:
//       use crate::components::subscription as sub;
//
//  2. Inside the `#[contractimpl] impl ShadeTrait for ShadeContract { ... }`
//     block, add these six delegating methods:
//
//     fn create_subscription_plan(env: Env, merchant: Address, token: Address, amount: i128, interval: u64) -> u64 {
//         sub::create_subscription_plan(&env, merchant, token, amount, interval)
//     }
//
//     fn get_subscription_plan(env: Env, plan_id: u64) -> SubscriptionPlan {
//         sub::get_subscription_plan(&env, plan_id)
//     }
//
//     fn subscribe(env: Env, customer: Address, plan_id: u64) -> u64 {
//         sub::subscribe(&env, customer, plan_id)
//     }
//
//     fn get_subscription(env: Env, subscription_id: u64) -> Subscription {
//         sub::get_subscription(&env, subscription_id)
//     }
//
//     fn charge_subscription(env: Env, subscription_id: u64) {
//         sub::charge_subscription(&env, subscription_id)
//     }
//
//     fn cancel_subscription(env: Env, caller: Address, subscription_id: u64) {
//         sub::cancel_subscription(&env, caller, subscription_id)
//     }
//
//  3. In contracts/shade/src/components/mod.rs, add:
//       pub mod subscription;
//
//  4. Add new error variants to errors.rs (see errors_additions.rs output).
// ============================================================================