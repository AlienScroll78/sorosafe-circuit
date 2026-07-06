# SoroSafe Circuit API Reference

## Contract Interface

### Initialization

#### `init(admin: Address)`

Initializes the contract with an admin account. Must be called before any other operations.

**Parameters:**
- `admin` (Address): The admin account that will control warden authorization and upgrades

**Authentication:** Requires `admin` to sign the transaction

**Events Emitted:** None (initialization event can be tracked via contract instantiation)

**Example:**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin_deployer \
  --network testnet \
  -- init \
  --admin GXXX...
```

**Error Cases:**
- None (first call succeeds, subsequent calls overwrite)

---

### Vault Management

#### `register_vault(vault, owner, evacuation, token, threshold)`

Registers a new vault for circuit breaker monitoring.

**Parameters:**
- `vault` (Address): The vault contract/account to monitor
- `owner` (Address): Principal vault owner (must authenticate)
- `evacuation` (Address): Safe address for emergency asset transfer
- `token` (Address): Soroban token contract address
- `threshold` (i128): Maximum allowed balance drop (in atomic units)

**Authentication:** Requires `owner` to sign the transaction

**Storage Impact:**
- Stores VaultConfig in persistent storage with 10,000-50,000 ledger TTL

**Events Emitted:**
```
Topic: ("SoroSafe", "Registered")
Data: (vault_address)
Ledger: Latest
```

**Example:**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source vault_owner \
  --network testnet \
  -- register_vault \
  --vault GVAULT123... \
  --owner GOWNER456... \
  --evacuation GEVAC789... \
  --token CTOKEN012... \
  --threshold 5000
```

**Error Cases:**
- `NotAuthorized` (401): Owner did not sign or did not authenticate
- Vault already registered: Overwrites previous configuration (no error)

**Notes:**
- Threshold should be calibrated based on expected operational patterns
- Vault address must be a valid Stellar account
- Token address must be a valid Soroban token contract
- Evacuation address should be a secure multi-sig or cold storage

---

### Warden Management

#### `set_warden(warden, status)`

Authorize or revoke a warden account's ability to trigger emergency stops.

**Parameters:**
- `warden` (Address): Warden account to authorize/revoke
- `status` (bool): true = authorize, false = revoke

**Authentication:** Requires `admin` to sign the transaction

**Storage Impact:**
- Stores boolean status in persistent storage

**Events Emitted:** None (can track via transaction history)

**Example (Add Warden):**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin_deployer \
  --network testnet \
  -- set_warden \
  --warden GWARDEN123... \
  --status true
```

**Example (Remove Warden):**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin_deployer \
  --network testnet \
  -- set_warden \
  --warden GWARDEN123... \
  --status false
```

**Error Cases:**
- `NotAuthorized` (401): Caller is not admin
- Warden not previously registered: Creates new entry (no error)

**Notes:**
- Admin can add/remove wardens at any time
- Wardens can only trigger emergency stops, not access funds
- Multiple wardens can be active simultaneously

---

### Contract Upgrade

#### `upgrade(new_wasm_hash)`

Upgrades the contract to a new WASM binary. Admin only.

**Parameters:**
- `new_wasm_hash` (BytesN<32>): SHA-256 hash of new WASM binary

**Authentication:** Requires `admin` to sign the transaction

**Storage Impact:**
- Preserves all vault configurations
- Preserves warden authorization
- Upgrades contract code

**Events Emitted:**
```
Topic: ("SoroSafe", "Upgraded")
Data: (admin_address)
Ledger: Upgrade ledger
```

**Example:**
```bash
# Compute WASM hash
SHA256=$(sha256sum contracts/sorosafe/target/wasm32-unknown-unknown/release/sorosafe_circuit.wasm | cut -d' ' -f1)

stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin_deployer \
  --network testnet \
  -- upgrade \
  --new_wasm_hash <32_byte_hash>
```

**Error Cases:**
- `NotAuthorized` (401): Caller is not admin
- Invalid hash format: Rejected by contract

**Notes:**
- New WASM must be audited before deployment
- All vault state is preserved
- After upgrade, contract is immediately operational
- Cannot downgrade to older WASM versions

---

### State Monitoring

#### `verify_state(vault, current_balance) -> bool`

Monitors vault state and detects balance anomalies. Core monitoring function.

**Parameters:**
- `vault` (Address): Vault to verify
- `current_balance` (i128): Current vault balance (in atomic units)

**Returns:** 
- `true` if vault is healthy (circuit closed)
- `false` if circuit is tripped (anomaly detected)

**Authentication:** No authentication required (read-like query, but mutates state)

**Storage Mutations:**
- Updates `last_balance` in VaultConfig
- Extends TTL for persistence
- May set `state` to Open if threshold exceeded

**Events Emitted (Conditional):**
```
Topic: ("SoroSafe", "Tripped_Auto")
Data: (vault_address)
Ledger: Current ledger
Condition: Balance drop > threshold
```

**Example:**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source monitoring_bot \
  --network testnet \
  -- verify_state \
  --vault GVAULT123... \
  --current_balance 95000
```

**Error Cases:**
- `VaultNotRegistered` (2): Vault not previously registered

**Logic:**
```
1. If vault not registered → panic(VaultNotRegistered)
2. If circuit already Open → return false
3. If (last_balance > 0 AND current_balance < last_balance):
     drain = last_balance - current_balance
     If drain > threshold:
        Set state = Open
        Emit Tripped_Auto event
        return false
4. Update last_balance = current_balance
5. Extend storage TTL
6. return true
```

**Notes:**
- Call this frequently (ideally on every operation)
- First call after registration will always return true
- Threshold comparison is strict (>) not (>=)
- TTL extension ensures vault isn't garbage collected

---

### Emergency Operations

#### `emergency_trip(caller, vault)`

Manually trigger emergency stop. Callable by owner or wardens only.

**Parameters:**
- `caller` (Address): The account triggering the trip (must be owner or warden)
- `vault` (Address): The vault to trip

**Authentication:** Requires `caller` to sign the transaction

**Storage Mutations:**
- Sets vault state to Open

**Events Emitted:**
```
Topic: ("SoroSafe", "Tripped_Manual")
Data: (vault, caller)
Ledger: Current ledger
```

**Example (Owner):**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source vault_owner \
  --network testnet \
  -- emergency_trip \
  --caller GOWNER456... \
  --vault GVAULT123...
```

**Example (Warden):**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source warden_account \
  --network testnet \
  -- emergency_trip \
  --caller GWARDEN123... \
  --vault GVAULT123...
```

**Error Cases:**
- `VaultNotRegistered` (2): Vault not previously registered
- `NotAuthorized` (1): Caller is neither owner nor authorized warden

**Notes:**
- No impact on vault funds (only state change)
- Subsequent `verify_state()` calls will return false
- Cannot reverse trip without re-registering vault
- Emit event for audit trail and monitoring

---

#### `execute_evacuation(vault)`

Transfer all vault assets to the emergency address. Only callable when circuit is open.

**Parameters:**
- `vault` (Address): The vault to evacuate

**Authentication:** No authentication required (anyone can execute)

**Storage Mutations:**
- No storage changes (state remains Open)

**Asset Transfer:**
- Transfers entire token balance from vault to evacuation address
- Requires prior approval from vault contract

**Events Emitted:**
```
Topic: ("SoroSafe", "Evacuated")
Data: (vault, amount_transferred)
Ledger: Current ledger
```

**Example:**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source any_account \
  --network testnet \
  -- execute_evacuation \
  --vault GVAULT123...
```

**Error Cases:**
- `VaultNotRegistered` (2): Vault not previously registered
- `CircuitNotOpen` (4): Circuit is not tripped (state = Closed)
- Token transfer fails: Propagates token contract error

**Token Transfer Semantics:**
```
token_client.transfer(
  from: vault,
  to: evacuation_address,
  amount: current_balance
)
```

**Notes:**
- Requires vault to have previously authorized circuit breaker for transfers
- Idempotent: Calling twice will transfer zero on second call
- Evacuation amount reported in event for auditability
- Should only be called after circuit is confirmed tripped

---

## Data Structures

### CircuitState
```rust
enum CircuitState {
    Closed,  // Normal operation
    Open,    // Emergency state
}
```

### VaultConfig
```rust
struct VaultConfig {
    owner: Address,              // Vault owner
    evacuation_address: Address, // Emergency transfer destination
    token_address: Address,      // Token contract
    state: CircuitState,         // Current state
    threshold_limit: i128,       // Max allowed drain
    last_balance: i128,          // Last known balance
}
```

### DataKey (Storage Keys)
```rust
enum DataKey {
    Config(Address),    // Map: vault -> VaultConfig
    Warden(Address),    // Map: warden -> is_authorized
}
```

---

## Error Codes

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 1 | NotAuthorized | Caller lacks required permissions | Verify authentication; use authorized account |
| 2 | VaultNotRegistered | Vault not found in contract | Register vault first |
| 3 | CircuitAlreadyOpen | Circuit is already tripped | N/A (informational) |
| 4 | CircuitNotOpen | Cannot evacuate without trip | Manually trip circuit first |
| 5 | InvalidBalanceDrop | Unused in current version | N/A |

---

## Event Schema

### Registered
```
Topic 0: "SoroSafe"
Topic 1: "Registered"
Data: vault_address
```

### Tripped_Auto
```
Topic 0: "SoroSafe"
Topic 1: "Tripped_Auto"
Data: vault_address
```

### Tripped_Manual
```
Topic 0: "SoroSafe"
Topic 1: "Tripped_Manual"
Data: (vault_address, caller_address)
```

### Evacuated
```
Topic 0: "SoroSafe"
Topic 1: "Evacuated"
Data: (vault_address, amount_transferred)
```

### Upgraded
```
Topic 0: "SoroSafe"
Topic 1: "Upgraded"
Data: admin_address
```

---

## Integration Examples

### Register and Monitor Vault

```bash
# 1. Register
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source owner \
  --network testnet \
  -- register_vault \
  --vault G... --owner G... --evacuation G... --token C... --threshold 5000

# 2. Monitor (repeat periodically)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- verify_state \
  --vault G... \
  --current_balance $(get_current_balance)

# 3. If exploit detected, trip and evacuate
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source owner_or_warden \
  --network testnet \
  -- emergency_trip \
  --caller G... \
  --vault G...

stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- execute_evacuation \
  --vault G...
```

### Warden Setup

```bash
# 1. Add primary warden
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- set_warden \
  --warden <WARDEN_1> \
  --status true

# 2. Add backup warden
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- set_warden \
  --warden <WARDEN_2> \
  --status true

# 3. Either warden can now emergency_trip independently
```

---

## Best Practices

### Threshold Selection

- **Conservative (1-5% of balance):** Triggers frequently but catches small exploits
- **Moderate (5-20% of balance):** Balanced sensitivity and noise
- **Aggressive (20%+ of balance):** Catches only large-scale exploits, less false positives

### Evacuation Address

- [ ] Use multi-sig account for additional safety
- [ ] Test transfers on testnet before production
- [ ] Ensure it's not frozen or restricted
- [ ] Consider timelocks for additional control

### Monitoring Frequency

- **High-security vaults:** Every transaction (inline calls)
- **Production vaults:** Every 60 seconds
- **Development vaults:** Every 5 minutes

### Emergency Response

1. Trip immediately when exploit detected
2. Assess damage within 5 minutes
3. Execute evacuation within 10 minutes if confirmed exploit
4. Notify stakeholders within 15 minutes

---

## Rate Limits & Costs

| Operation | Cost (in Stroops) | Typical Frequency |
|-----------|-------------------|-------------------|
| register_vault | ~5,000 | Once per vault |
| verify_state | ~500 | Every transaction |
| emergency_trip | ~1,000 | Rare |
| execute_evacuation | ~5,000 | Rare |
| set_warden | ~1,000 | Rare |

---

## Network Endpoints

**Testnet:**
```
RPC: https://soroban-testnet.stellar.org:443
Network Passphrase: Test SDF Network ; September 2015
```

**Mainnet:**
```
RPC: https://soroban-mainnet.stellar.org
Network Passphrase: Public Global Stellar Network ; September 2015
```

---

## Further Documentation

- **Architecture:** `docs/ARCHITECTURE.md`
- **Deployment:** `docs/DEPLOYMENT.md`
- **Security:** `docs/SECURITY.md`
- **Operations:** `docs/OPERATIONS.md`
- **Testing:** `docs/TESTING.md`
