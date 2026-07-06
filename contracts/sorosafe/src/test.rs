#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env, Address};

#[test]
fn test_circuit_lifecycle_and_trip() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, SoroSafeCircuit);
    let client = SoroSafeCircuitClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vault = Address::generate(&env);
    let owner = Address::generate(&env);
    let evacuation = Address::generate(&env);
    let token = Address::generate(&env);

    // Initialize contract
    client.init(&admin);

    // Register vault with 500 unit threshold
    client.register_vault(&vault, &owner, &evacuation, &token, &500_i128);

    // Initial check: normal balance
    let is_valid = client.verify_state(&vault, &1000_i128);
    assert!(is_valid);

    // Drop within threshold: 200 units drop (below 500 threshold)
    let process_drop = client.verify_state(&vault, &800_i128);
    assert!(process_drop);

    // Massive exploit drop: 800 units (exceeds 500 threshold) should trip circuit
    let exploit_drop = client.verify_state(&vault, &200_i128);
    assert!(!exploit_drop);
}

#[test]
fn test_warden_emergency_trip() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, SoroSafeCircuit);
    let client = SoroSafeCircuitClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vault = Address::generate(&env);
    let owner = Address::generate(&env);
    let evacuation = Address::generate(&env);
    let token = Address::generate(&env);
    let warden = Address::generate(&env);

    client.init(&admin);
    client.register_vault(&vault, &owner, &evacuation, &token, &500_i128);
    client.set_warden(&warden, true);

    // Warden can manually trip the circuit
    client.emergency_trip(&warden, &vault);

    // After trip, verify_state should return false
    let is_valid = client.verify_state(&vault, &1000_i128);
    assert!(!is_valid);
}

#[test]
#[should_panic]
fn test_unauthorized_access() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, SoroSafeCircuit);
    let client = SoroSafeCircuitClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vault = Address::generate(&env);
    let owner = Address::generate(&env);
    let evacuation = Address::generate(&env);
    let token = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    client.init(&admin);
    client.register_vault(&vault, &owner, &evacuation, &token, &500_i128);

    // Unauthorized user tries to trip the circuit
    client.emergency_trip(&unauthorized, &vault);
}

#[test]
fn test_vault_not_registered() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, SoroSafeCircuit);
    let client = SoroSafeCircuitClient::new(&env, &contract_id);

    let vault = Address::generate(&env);

    // Verify state on non-existent vault should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.verify_state(&vault, &1000_i128);
    }));

    assert!(result.is_err());
}
