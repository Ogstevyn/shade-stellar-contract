use crate::errors::ContractError;
use crate::events::publish_nonce_invalidated_event;
use crate::types::DataKey;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{panic_with_error, Address, Bytes, BytesN, Env, String};

/// Builds the message that the merchant must have signed.
///
/// Format: [contract_address, merchant_address, nonce, amount, token_address, description_bytes]
fn build_message(
    env: &Env,
    merchant: &Address,
    description: &String,
    amount: i128,
    token: &Address,
    nonce: &BytesN<32>,
) -> Bytes {
    let mut msg = Bytes::new(env);
    msg.append(&env.current_contract_address().to_xdr(env));
    msg.append(&merchant.clone().to_xdr(env));
    msg.append(nonce.as_ref());
    msg.append(&Bytes::from_slice(env, &amount.to_be_bytes()));
    msg.append(&token.clone().to_xdr(env));
    msg.append(&description.clone().to_xdr(env));
    msg
}

/// Verifies the merchant's ed25519 signature over the invoice parameters.
///
/// Retrieves the merchant's public key from `DataKey::MerchantKey(merchant)`.
/// Panics with `MerchantKeyNotFound` if the key is not registered.
/// Panics with a crypto error if the signature is invalid.
pub fn verify_invoice_signature(
    env: &Env,
    merchant: &Address,
    description: &String,
    amount: i128,
    token: &Address,
    nonce: &BytesN<32>,
    signature: &BytesN<64>,
) {
    let key: BytesN<32> = env
        .storage()
        .persistent()
        .get(&DataKey::MerchantKey(merchant.clone()))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::MerchantKeyNotFound));

    let message = build_message(env, merchant, description, amount, token, nonce);

    env.crypto().ed25519_verify(&key, &message, signature);
}

/// Checks the nonce has not been used and marks it as used.
///
/// Panics with `NonceAlreadyUsed` if the nonce was already consumed.
/// Emits a `NonceInvalidated` event on success.
pub fn invalidate_nonce(env: &Env, merchant: &Address, nonce: &BytesN<32>) {
    let nonce_key = DataKey::UsedNonce(merchant.clone(), nonce.clone());

    if env.storage().persistent().has(&nonce_key) {
        panic_with_error!(env, ContractError::NonceAlreadyUsed);
    }

    env.storage().persistent().set(&nonce_key, &true);

    publish_nonce_invalidated_event(
        env,
        merchant.clone(),
        nonce.clone(),
        env.ledger().timestamp(),
    );
}
