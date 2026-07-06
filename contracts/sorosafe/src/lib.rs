#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, Address, Env, Symbol, BytesN, panic_with_error};

mod types;

#[cfg(test)]
mod test;

use types::{CircuitState, VaultConfig, DataKey};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CircuitError {
    NotAuthorized = 1,
    VaultNotRegistered = 2,
    CircuitAlreadyOpen = 3,
    CircuitNotOpen = 4,
    InvalidBalanceDrop = 5,
}

#[contract]
pub struct SoroSafeCircuit;

#[contractimpl]
impl SoroSafeCircuit {
    /// Initialize the circuit breaker with an admin account.
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "admin"), &admin);
    }

    /// Upgrade the contract to a new WASM hash. Only callable by admin.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap();
        admin.require_auth();
        env.deployer()
            .update_current_contract_wasm(new_wasm_hash);
        env.events().publish(
            (Symbol::new(&env, "SoroSafe"), Symbol::new(&env, "Upgraded")),
            admin,
        );
    }

    /// Register a new vault with the circuit breaker.
    pub fn register_vault(
        env: Env,
        vault: Address,
        owner: Address,
        evacuation: Address,
        token: Address,
        threshold: i128,
    ) {
        owner.require_auth();

        let config = VaultConfig {
            owner: owner.clone(),
            evacuation_address: evacuation,
            token_address: token,
            state: CircuitState::Closed,
            threshold_limit: threshold,
            last_balance: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Config(vault.clone()), &config);

        env.events().publish(
            (Symbol::new(&env, "SoroSafe"), Symbol::new(&env, "Registered")),
            vault,
        );
    }

    /// Designate a warden address with emergency trip authority.
    pub fn set_warden(env: Env, warden: Address, status: bool) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .unwrap();
        admin.require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::Warden(warden), &status);
    }

    /// Verify vault state and detect balance anomalies.
    /// Returns true if state is valid, false if circuit should be tripped.
    pub fn verify_state(env: Env, vault: Address, current_balance: i128) -> bool {
        let key = DataKey::Config(vault.clone());

        if !env.storage().persistent().has(&key) {
            panic_with_error!(&env, CircuitError::VaultNotRegistered);
        }

        let mut config: VaultConfig = env.storage().persistent().get(&key).unwrap();

        // If circuit is already open, return false immediately.
        if let CircuitState::Open = config.state {
            return false;
        }

        // Check for unusual balance drops.
        if config.last_balance > 0 && current_balance < config.last_balance {
            let drain_amount = config.last_balance - current_balance;
            if drain_amount > config.threshold_limit {
                config.state = CircuitState::Open;
                env.storage().persistent().set(&key, &config);
                env.events().publish(
                    (Symbol::new(&env, "SoroSafe"), Symbol::new(&env, "Tripped_Auto")),
                    vault,
                );
                return false;
            }
        }

        // Update balance and extend TTL for persistence.
        config.last_balance = current_balance;
        env.storage().persistent().set(&key, &config);
        env.storage()
            .persistent()
            .extend_ttl(&key, 10000, 50000);

        true
    }

    /// Manually trip the circuit. Callable by owner or authorized wardens.
    pub fn emergency_trip(env: Env, caller: Address, vault: Address) {
        caller.require_auth();

        let key = DataKey::Config(vault.clone());

        if !env.storage().persistent().has(&key) {
            panic_with_error!(&env, CircuitError::VaultNotRegistered);
        }

        let mut config: VaultConfig = env.storage().persistent().get(&key).unwrap();

        let is_warden: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Warden(caller.clone()))
            .unwrap_or(false);

        if caller != config.owner && !is_warden {
            panic_with_error!(&env, CircuitError::NotAuthorized);
        }

        config.state = CircuitState::Open;
        env.storage().persistent().set(&key, &config);

        env.events().publish(
            (Symbol::new(&env, "SoroSafe"), Symbol::new(&env, "Tripped_Manual")),
            (vault, caller),
        );
    }

    /// Execute evacuation of vault assets to the emergency address.
    /// Only callable when circuit is tripped (state = Open).
    pub fn execute_evacuation(env: Env, vault: Address) {
        let key = DataKey::Config(vault.clone());

        if !env.storage().persistent().has(&key) {
            panic_with_error!(&env, CircuitError::VaultNotRegistered);
        }

        let config: VaultConfig = env.storage().persistent().get(&key).unwrap();

        if let CircuitState::Closed = config.state {
            panic_with_error!(&env, CircuitError::CircuitNotOpen);
        }

        let token_client = soroban_sdk::token::Client::new(&env, &config.token_address);
        let current_pool_balance = token_client.balance(&vault);

        if current_pool_balance > 0 {
            token_client.transfer(
                &vault,
                &config.evacuation_address,
                &current_pool_balance,
            );

            env.events().publish(
                (Symbol::new(&env, "SoroSafe"), Symbol::new(&env, "Evacuated")),
                (vault, current_pool_balance),
            );
        }
    }
}
