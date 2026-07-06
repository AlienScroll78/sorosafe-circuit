# SoroSafe Testing Guide

## Unit Tests

Located in `contracts/sorosafe/src/test.rs`

Run all tests:
```bash
cd contracts/sorosafe && cargo test
```

Run specific test:
```bash
cargo test test_circuit_lifecycle_and_trip -- --nocapture
```

### Test Coverage

#### 1. Circuit Lifecycle Test
**File:** `test_circuit_lifecycle_and_trip`

Tests the complete state machine:
- Initialize contract with admin
- Register vault with threshold
- Query state at different balance levels
- Verify automatic trip when threshold exceeded

**Expected Behavior:**
- Initial check returns true (healthy)
- Drop within threshold returns true (healthy)
- Drop exceeding threshold returns false (circuit tripped)

**Critical Assertions:**
```rust
assert!(is_valid);           // Normal balance
assert!(process_drop);        // Minor drop
assert!(!exploit_drop);       // Major drop triggers trip
```

#### 2. Warden Authorization Test
**File:** `test_warden_emergency_trip`

Tests warden-initiated emergency stops:
- Set warden status
- Warden calls emergency_trip
- Verify state returns false after trip

**Expected Behavior:**
- Warden can trip circuit without owner signature
- Circuit state changes to Open
- Subsequent state queries return false

#### 3. Unauthorized Access Test
**File:** `test_unauthorized_access`

Tests access control enforcement:
- Unauthorized account attempts to trip circuit
- Should panic with NotAuthorized error

**Expected Behavior:**
- Non-owner, non-warden cannot trip circuit
- Panic prevents state mutation

#### 4. Vault Registration Test
**File:** `test_vault_not_registered`

Tests error handling for unregistered vaults:
- Query state on non-existent vault
- Should panic with VaultNotRegistered error

**Expected Behavior:**
- Unregistered vault queries fail fast
- Clear error messages for debugging

## Integration Testing

### Manual Testnet Scenario

**Objective:** Verify end-to-end workflow with real Soroban network

**Prerequisites:**
- Contract deployed to testnet
- Testnet XLM available
- Monitoring bot running

**Steps:**

1. **Register Vault**
   ```bash
   VAULT="G..."
   OWNER="GOWNER..."
   EVAC="GEVAC..."
   TOKEN="CTOKEN..."
   
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --source $OWNER \
     --network testnet \
     -- register_vault \
     --vault $VAULT \
     --owner $OWNER \
     --evacuation $EVAC \
     --token $TOKEN \
     --threshold 1000
   ```

2. **Verify Initial State**
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --network testnet \
     -- verify_state \
     --vault $VAULT \
     --current_balance 50000
   # Should return true
   ```

3. **Simulate Balance Anomaly**
   ```bash
   # Simulate balance drop > threshold
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --network testnet \
     -- verify_state \
     --vault $VAULT \
     --current_balance 48000  # Drop of 2000 > 1000 threshold
   # Should return false (circuit tripped)
   ```

4. **Execute Evacuation**
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --source operator \
     --network testnet \
     -- execute_evacuation \
     --vault $VAULT
   ```

5. **Verify Outcome**
   - Check events emitted
   - Confirm evacuation address received tokens
   - Query vault state (should still be Open)

### Multi-Vault Concurrent Testing

**Objective:** Verify independent vault management

**Setup:**
```bash
# Register 3 separate vaults
VAULT1="G1..."
VAULT2="G2..."
VAULT3="G3..."

for VAULT in $VAULT1 $VAULT2 $VAULT3; do
  stellar contract invoke \
    --id <CONTRACT_ID> \
    --source $OWNER \
    --network testnet \
    -- register_vault \
    --vault $VAULT \
    --owner $OWNER \
    --evacuation $EVAC \
    --token $TOKEN \
    --threshold 1000
done
```

**Test:**
- Trip only Vault1
- Verify Vault2 and Vault3 remain healthy
- Execute evacuation for Vault1 only
- Confirm evacuation doesn't affect other vaults

### Warden Multi-Sig Testing

**Objective:** Verify warden-based emergency stops

**Setup:**
```bash
WARDEN1="GW1..."
WARDEN2="GW2..."

stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- set_warden \
  --warden $WARDEN1 \
  --status true

stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- set_warden \
  --warden $WARDEN2 \
  --status true
```

**Test:**
- Either warden can trip circuit independently
- Neither requires owner signature
- Both trips result in same circuit state

## Performance Testing

### Contract Size

Current WASM size should be minimal:

```bash
ls -lh contracts/sorosafe/target/wasm32-unknown-unknown/release/sorosafe_circuit.wasm
# Expected: ~10-20KB
```

If WASM exceeds 50KB, investigate for bloat:
```bash
cargo bloat --release -n 10
```

### State Query Latency

Measure `verify_state` execution time:

```bash
# Timestamp before query
START=$(date +%s%N)

stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- verify_state \
  --vault <vault> \
  --current_balance <balance>

# Timestamp after response
END=$(date +%s%N)

LATENCY_MS=$(( (END - START) / 1000000 ))
echo "Query latency: ${LATENCY_MS}ms"
```

**Target:** < 1000ms for typical RPC providers

### Evacuation Throughput

Test evacuation with varying token amounts:

```bash
# Small amount (10 tokens)
time stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- execute_evacuation \
  --vault <vault_with_small_balance>

# Large amount (1M tokens)
time stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- execute_evacuation \
  --vault <vault_with_large_balance>
```

**Expected:** Latency remains constant regardless of amount (O(1) operation)

## Security Testing

### Authorization Testing

Verify access control with unauthorized accounts:

```bash
# Attempt to set warden as non-admin
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source unauthorized_account \
  --network testnet \
  -- set_warden \
  --warden <some_address> \
  --status true
# Should fail with NotAuthorized
```

### Threshold Boundary Testing

Test extreme threshold values:

```bash
# Minimum threshold (0)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source $OWNER \
  --network testnet \
  -- register_vault \
  --vault <test_vault_1> \
  --owner $OWNER \
  --evacuation $EVAC \
  --token $TOKEN \
  --threshold 0
# Verify any balance drop triggers circuit

# Maximum threshold (i128::MAX)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source $OWNER \
  --network testnet \
  -- register_vault \
  --vault <test_vault_2> \
  --owner $OWNER \
  --evacuation $EVAC \
  --token $TOKEN \
  --threshold 9223372036854775807
# Verify no reasonable balance drop triggers circuit
```

### Token Integration Testing

Verify token contract interaction:

```bash
# Test with different token types
# 1. Standard token (18 decimals)
# 2. Non-standard token (6 decimals)
# 3. Frozen token account (should fail evacuation)

stellar contract invoke \
  --id <CONTRACT_ID> \
  --source operator \
  --network testnet \
  -- execute_evacuation \
  --vault <vault_with_frozen_token>
# Should fail with token contract error
```

## Regression Testing

Create a regression suite to run before each deployment:

```bash
#!/bin/bash
# scripts/regression_test.sh

CONTRACT_ID="$1"
NETWORK="${2:-testnet}"

echo "Running regression tests on $CONTRACT_ID ($NETWORK)..."

# Test 1: Init contract
echo "Test 1: Init..."
# ... test code

# Test 2: Register vault
echo "Test 2: Register vault..."
# ... test code

# Test 3: Verify state
echo "Test 3: Verify state..."
# ... test code

# Test 4: Emergency trip
echo "Test 4: Emergency trip..."
# ... test code

# Test 5: Evacuation
echo "Test 5: Evacuation..."
# ... test code

echo "✓ All regression tests passed"
```

Run before mainnet deployment:
```bash
./scripts/regression_test.sh <CONTRACT_ID> mainnet
```

## Testing Checklist

- [ ] All unit tests pass: `cargo test`
- [ ] Contract compiles without warnings
- [ ] WASM size is reasonable (< 50KB)
- [ ] Manual testnet scenario completes successfully
- [ ] Multi-vault isolation verified
- [ ] Warden authorization works correctly
- [ ] Unauthorized access is properly blocked
- [ ] Evacuation transfers correct amount
- [ ] Event emissions are accurate
- [ ] Boundary values (0, i128::MAX) handled correctly
- [ ] Token integration tested with real token
- [ ] Emergency trip from multiple wardens verified
- [ ] Contract upgrade doesn't lose vault state
- [ ] Monitoring alerts fire on trip events
- [ ] Recovery procedures documented and tested

## Continuous Integration

Recommended CI/CD checks:

```yaml
# Run on every commit
- cargo test
- cargo clippy
- cargo fmt --check

# Run on every PR to main
- Full integration test suite
- Testnet deployment smoke test
- Security audit (if available)

# Run before mainnet release
- Full regression test suite
- Performance benchmarks
- Manual security review
```

## Test Fixtures

Useful test data for replication:

```rust
#[cfg(test)]
mod fixtures {
    use super::*;

    pub const THRESHOLD_CONSERVATIVE: i128 = 500;
    pub const THRESHOLD_MODERATE: i128 = 5_000;
    pub const THRESHOLD_AGGRESSIVE: i128 = 50_000;

    pub const BALANCE_NORMAL: i128 = 1_000_000;
    pub const BALANCE_DEPLETED: i128 = 100_000;
    pub const BALANCE_EMPTY: i128 = 0;

    pub fn generate_test_vault() -> (Address, Address, Address) {
        let vault = Address::generate(&env);
        let owner = Address::generate(&env);
        let evacuation = Address::generate(&env);
        (vault, owner, evacuation)
    }
}
```

## Further Reading

- Soroban Testing: https://developers.stellar.org/docs/build/smart-contracts/testing
- Property-Based Testing: https://hypothesis.readthedocs.io/
- Fuzzing: https://rust-fuzz.github.io/
