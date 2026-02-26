#![cfg(test)]

use crate::shade::Shade;
use crate::shade::ShadeClient;
use crate::types::Role;
use soroban_sdk::testutils::{Address as _, Events, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, FromVal, IntoVal, Symbol};

fn setup_test(env: &Env) -> (ShadeClient<'_>, Address) {
    let contract_id = env.register(Shade, ());
    let client = ShadeClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

#[test]
fn test_grant_role() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_test(&env);
    let user = Address::generate(&env);

    assert!(!client.has_role(&user, &Role::Manager));

    client.grant_role(&admin, &user, &Role::Manager);

    assert!(client.has_role(&user, &Role::Manager));
}

#[test]
fn test_revoke_role() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_test(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Manager);
    assert!(client.has_role(&user, &Role::Manager));

    client.revoke_role(&admin, &user, &Role::Manager);
    assert!(!client.has_role(&user, &Role::Manager));
}

#[test]
fn test_admin_supremacy() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_test(&env);

    // Admin should have all roles implicitly
    assert!(client.has_role(&admin, &Role::Manager));
    assert!(client.has_role(&admin, &Role::Operator));
    assert!(client.has_role(&admin, &Role::Admin));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")] // NotAuthorized
fn test_unauthorized_grant() {
    let env = Env::default();
    let (client, _admin) = setup_test(&env);
    let user = Address::generate(&env);
    let malicious = Address::generate(&env);

    // Mock only malicious user's auth, but they aren't admin
    env.mock_auths(&[MockAuth {
        address: &malicious,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: (&malicious, &user, Role::Manager).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.grant_role(&malicious, &user, &Role::Manager);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")] // NotAuthorized
fn test_unauthorized_revoke() {
    let env = Env::default();
    let (client, admin) = setup_test(&env);
    let user = Address::generate(&env);
    let malicious = Address::generate(&env);

    // Grant first as admin
    env.mock_all_auths();
    client.grant_role(&admin, &user, &Role::Manager);

    // Try to revoke as malicious user
    env.mock_auths(&[MockAuth {
        address: &malicious,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_role",
            args: (&malicious, &user, Role::Manager).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.revoke_role(&malicious, &user, &Role::Manager);
}

#[test]
fn test_events() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_test(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Manager);

    let events = env.events().all();
    let last_event = events.last().unwrap();

    assert_eq!(last_event.0, client.address);
    let first_topic = Symbol::from_val(&env, &last_event.1.get(0).unwrap());
    assert_eq!(first_topic, Symbol::new(&env, "role_granted_event"));

    assert!(!events.is_empty());

    client.revoke_role(&admin, &user, &Role::Manager);

    let events = env.events().all();
    let last_event = events.last().unwrap();

    let first_topic = Symbol::from_val(&env, &last_event.1.get(0).unwrap());
    assert_eq!(last_event.0, client.address);
    assert_eq!(first_topic, Symbol::new(&env, "role_revoked_event"));
}

#[test]
fn test_duplicate_grant() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_test(&env);
    let user = Address::generate(&env);

    client.grant_role(&admin, &user, &Role::Manager);
    assert!(client.has_role(&user, &Role::Manager));

    // Granting again should not change anything or panic
    client.grant_role(&admin, &user, &Role::Manager);
    assert!(client.has_role(&user, &Role::Manager));
}

#[test]
fn test_revoke_non_existent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_test(&env);
    let user = Address::generate(&env);

    // Revoking a role that wasn't granted should not panic
    client.revoke_role(&admin, &user, &Role::Manager);
    assert!(!client.has_role(&user, &Role::Manager));
}
