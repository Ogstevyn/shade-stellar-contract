#![cfg(test)]

use crate::shade::{Shade, ShadeClient};
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal};

fn setup_test(env: &Env) -> (ShadeClient<'_>, Address) {
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

#[test]
fn test_admin_transfer_successful() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_test(&env);
    let new_admin = Address::generate(&env);

    client.propose_admin_transfer(&admin, &new_admin);
    assert_eq!(client.get_admin(), admin);

    client.accept_admin_transfer(&new_admin);
    assert_eq!(client.get_admin(), new_admin);
}

#[test]
fn test_admin_transfer_unauthorized_proposal() {
    let env = Env::default();
    let (client, _admin) = setup_test(&env);
    let malicious = Address::generate(&env);
    let new_admin = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &malicious,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "propose_admin_transfer",
            args: (&malicious, &new_admin).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let result = client.try_propose_admin_transfer(&malicious, &new_admin);
    assert!(result.is_err());
}

#[test]
fn test_admin_transfer_unauthorized_acceptance() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_test(&env);
    let new_admin = Address::generate(&env);
    let malicious = Address::generate(&env);

    client.propose_admin_transfer(&admin, &new_admin);

    let result = client.try_accept_admin_transfer(&malicious);
    assert!(result.is_err());
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_admin_transfer_overwrite_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_test(&env);
    let first = Address::generate(&env);
    let second = Address::generate(&env);

    client.propose_admin_transfer(&admin, &first);
    client.propose_admin_transfer(&admin, &second);

    let result = client.try_accept_admin_transfer(&first);
    assert!(result.is_err());

    client.accept_admin_transfer(&second);
    assert_eq!(client.get_admin(), second);
}
