use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CircuitState {
    Closed, // Normal operation (monitoring)
    Open,   // Emergency state (tripped)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultConfig {
    pub owner: Address,
    pub evacuation_address: Address,
    pub token_address: Address,
    pub state: CircuitState,
    pub threshold_limit: i128,
    pub last_balance: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Config(Address),   // Maps Vault Address -> VaultConfig
    Warden(Address),   // Maps Warden Address -> IsActive Boolean
}
