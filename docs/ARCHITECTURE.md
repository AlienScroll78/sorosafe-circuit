# SoroSafe Circuit Architecture

## Overview

SoroSafe is a zero-trust circuit breaker system for Soroban vaults. It provides three core mechanisms:

1. **State Verification** - Continuous monitoring of vault balances against configurable thresholds
2. **Emergency Trip** - Manual override capability for vault owners and designated wardens
3. **Evacuation** - Atomic asset transfer to a pre-configured safe address when circuit opens

## Design Principles

### Explicit Authentication
All state-mutating operations require explicit authentication via `require_auth()`. This prevents unauthorized calls and ensures every action is cryptographically signed by the originating account.

### Event Emission
Comprehensive event publishing enables off-chain indexing and monitoring:
- `Registered` - Vault registration
- `Tripped_Auto` - Automatic threshold detection
- `Tripped_Manual` - Owner/warden manual override
- `Evacuated` - Emergency asset transfer
- `Upgraded` - Contract WASM upgrade

### Persistent Storage with TTL Management
Vault configurations are stored in persistent storage with dynamic TTL extensions to ensure availability during monitoring windows.

### Warden Authorization Model
The owner designates trusted wardens who can independently trigger emergency stops without requiring owner authentication for every action.

## State Machine

```
┌─────────┐  verify_state detects drain  ┌──────┐
│ CLOSED  │─────────────────────────────>│ OPEN │
│         │<─────────────────────────────│      │
└─────────┘  manual emergency_trip()     └──────┘
    ▲                                        │
    └────────────────────────────────────────┘
         evacuate assets (one-way)
```

### CLOSED State
- Normal operation
- `verify_state()` monitors balance changes
- If drain > threshold, transitions to OPEN
- Wardens can manually trigger trip

### OPEN State
- Circuit breaker is engaged
- `verify_state()` returns false (blocks new operations)
- `execute_evacuation()` transfers all remaining assets
- Cannot transition back (requires new vault registration)

## Key Functions

### `init(admin: Address)`
Initializes the contract with an admin account. Admin controls warden authorization and contract upgrades.

### `register_vault(vault, owner, evacuation, token, threshold)`
Registers a new vault for monitoring. Requires owner authentication.
- `vault` - The vault account address
- `owner` - Principal account (must authenticate to register)
- `evacuation` - Safe address for emergency asset transfer
- `token` - Token contract being monitored
- `threshold` - Maximum allowed balance drop in atomic unit

### `set_warden(warden, status)`
Designates or revokes warden status. Only admin can call.

### `verify_state(vault, current_balance) -> bool`
Called periodically to monitor vault health. Returns true if vault is healthy, false if circuit should be tripped.

Logic:
1. If vault not registered → panic
2. If circuit already open → return false
3. If balance drop > threshold → open circuit and emit event
4. Update last known balance and extend TTL
5. Return true if healthy

### `emergency_trip(caller, vault)`
Manual override to immediately trip circuit. Callable by owner or authorized wardens.

### `execute_evacuation(vault)`
Transfers all vault assets to the evacuation address. Only callable when circuit is open.

## Token Integration

SoroSafe uses Soroban's standard token interface:
```rust
let token_client = soroban_sdk::token::Client::new(&env, &config.token_address);
let balance = token_client.balance(&vault);
token_client.transfer(&vault, &evacuation_address, &balance);
```

The vault must have pre-authorized the circuit breaker contract for token transfers (via allowance or delegation).

## Security Considerations

### Threshold Calibration
Choose thresholds based on:
- Normal operational volatility
- Expected transaction sizes
- False-positive tolerance

Too low → frequent false triggers. Too high → inadequate protection.

### Evacuation Address Selection
- Should be a secure multi-sig or cold storage account
- Test thoroughly on testnet before mainnet deployment
- Consider timelocks or additional safeguards at the destination

### Warden Trust Model
- Wardens can unilaterally trip the circuit but cannot access vault funds
- Design for Byzantine fault tolerance if multiple wardens
- Regular audits of warden authorization status

### TTL Management
Storage entries are extended to 10,000-50,000 ledger slots (~12-60 days on production).
For long-term vaults, implement periodic `verify_state()` calls to refresh TTL.

## Event Indexing

All circuit events are published with consistent structure:
```
(namespace: "SoroSafe", event_type: "Tripped_Auto"|"Tripped_Manual"|etc.)
Event data: (vault_address, [caller], [amount])
```

Off-chain indexers can subscribe to these events to:
- Alert operators of circuit trips
- Maintain audit logs
- Trigger downstream evacuation coordination

## Testing

The contract includes comprehensive unit tests:
- Lifecycle transitions
- Threshold violation detection
- Authorization enforcement
- Warden functionality
- Error handling

Run tests:
```bash
cargo test --all
```

## Deployment Checklist

- [ ] Contract compiled and tested on testnet
- [ ] Admin account funded and secured
- [ ] Vault owner accounts prepared
- [ ] Evacuation address confirmed (multi-sig recommended)
- [ ] Token contracts verified
- [ ] Threshold values calibrated for expected operations
- [ ] Warden accounts identified and secured
- [ ] Monitoring/indexing infrastructure ready
- [ ] Mainnet deployment approved by security review
