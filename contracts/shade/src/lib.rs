#![no_std]
use soroban_sdk::{contract, contractimpl, vec, Env, String, Vec};

#[contract]
pub struct Shade;

#[contractimpl]
impl Shade {
    pub fn hello_world(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "Hello World"), to]
    }
}

mod test;
