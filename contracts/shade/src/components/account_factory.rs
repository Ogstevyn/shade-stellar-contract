use crate::events;
use soroban_sdk::{Address, Bytes, BytesN, Env, IntoVal};

pub fn deploy_account(
    env: &Env,
    merchant: Address,
    manager: Address,
    merchant_id: u64,
    wasm_hash: BytesN<32>,
) -> Address {
    // Generate a random salt for deployment.
    let random_bytes_n: BytesN<32> = env.prng().gen();
    let random_bytes = Bytes::from_slice(env, &random_bytes_n.to_array());
    let salt = env.crypto().keccak256(&random_bytes);

    let deployed_contract = env
        .deployer()
        .with_current_contract(salt)
        .deploy_v2(wasm_hash, ());

    // Initialize the deployed contract with the required arguments.
    // The account contract's `initialize` function signature is:
    // fn initialize(env: Env, merchant: Address, manager: Address, merchant_id: u64);
    env.invoke_contract::<()>(
        &deployed_contract,
        &soroban_sdk::Symbol::new(env, "initialize"),
        (merchant.clone(), manager, merchant_id).into_val(env),
    );

    events::publish_merchant_account_deployed_event(
        env,
        merchant,
        deployed_contract.clone(),
        env.ledger().timestamp(),
    );

    deployed_contract
}
