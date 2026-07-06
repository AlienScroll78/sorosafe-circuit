# SoroSafe Operations Guide

## Overview

This guide covers day-to-day operational procedures for managing SoroSafe circuits in production.

## Daily Operations

### 1. Monitoring Vault Health

Set up continuous monitoring with this query pattern:

```bash
# Check vault state periodically (e.g., every 5 minutes)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source monitoring_bot \
  --network testnet \
  -- verify_state \
  --vault <vault_address> \
  --current_balance <latest_balance>
```

**Decision Tree:**
- If returns `true` → Vault is healthy, continue monitoring
- If returns `false` → **CIRCUIT IS TRIPPED**, proceed to "Emergency Response"

### 2. Balance Monitoring Setup

Recommended monitoring frequency: **Every transaction or every 60 seconds**, whichever is more frequent.

For high-frequency vaults:
```bash
# Monitor on every operation
before_operation: verify_state()
```

For lower-frequency vaults:
```bash
# Monitor periodically
cron: verify_state() every 5 minutes
```

### 3. Event Indexing

Subscribe to SoroSafe events via an off-chain indexer:

```
Event Channel: soroban-events
Filters:
  - contract_id = <CONTRACT_ID>
  - topics[0] = "SoroSafe"
  - topics[1] IN ["Registered", "Tripped_Auto", "Tripped_Manual", "Evacuated"]
```

Set up alerts for:
- **Tripped_Auto** → Anomaly detected, possible exploit
- **Tripped_Manual** → Emergency stop triggered
- **Registered** → New vault added (audit)

## Emergency Response

### Alert: Circuit Tripped

**Trigger:** `Tripped_Auto` or `Tripped_Manual` event

**Immediate Actions (0-5 minutes):**

1. **Verify Trip**
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --network testnet \
     -- verify_state \
     --vault <vault_address> \
     --current_balance <current_balance>
   # Should return false
   ```

2. **Investigate Cause**
   - Check vault transaction history
   - Query balance history from blockchain
   - Look for anomalous transfers
   - Check if this is a `Tripped_Manual` (owner action) or `Tripped_Auto` (exploit)

3. **Notify Stakeholders**
   - Alert vault owner
   - Notify operations team
   - Log incident with timestamp and reason

**Actions (5-30 minutes):**

4. **Decide: Execute Evacuation?**
   - If exploit confirmed → Execute evacuation immediately
   - If false positive → Investigate threshold and re-register vault

5. **If Executing Evacuation:**
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --source operator \
     --network testnet \
     -- execute_evacuation \
     --vault <vault_address>
   ```
   
   Verify evacuation:
   - Check `Evacuated` event shows expected amount
   - Confirm evacuation address received funds
   - Verify vault balance is now zero

6. **If False Positive:**
   - Analyze why threshold was exceeded
   - Recalibrate threshold if needed
   - Register new vault with updated settings
   - Document incident

### Alert: Unauthorized Trip

**Trigger:** `Tripped_Manual` event from unexpected account

**Immediate Actions:**

1. **Identify Caller**
   ```bash
   # Check event data
   Event: Tripped_Manual
   Data: (vault: <vault>, caller: <unexpected_address>)
   ```

2. **Verify Warden Status**
   ```bash
   # Query if caller is authorized warden
   stellar keys info <caller_address>
   ```

3. **If Unauthorized:**
   - Assume warden key compromise
   - Rotate warden keys immediately
   - Invalidate compromised warden:
     ```bash
     stellar contract invoke \
       --id <CONTRACT_ID> \
       --source admin \
       --network testnet \
       -- set_warden \
       --warden <compromised_address> \
       --status false
     ```

4. **Coordinate Response:**
   - Notify affected parties
   - Document incident
   - Re-register vault with new wardens if needed
   - Post-incident review

## Routine Maintenance

### Weekly

- [ ] Review and acknowledge all events
- [ ] Verify evacuation address is still correct/accessible
- [ ] Check warden key status and rotation schedule
- [ ] Review monitoring alerting health

### Monthly

- [ ] Audit vault configurations
- [ ] Review and validate threshold settings
- [ ] Test recovery procedures (if safe)
- [ ] Review event logs for anomalies
- [ ] Update documentation if needed

### Quarterly

- [ ] Full security audit
- [ ] Penetration testing (if using external contractors)
- [ ] Warden key rotation
- [ ] Disaster recovery drill
- [ ] Review and update incident response playbooks

## Configuration Changes

### Updating Vault Threshold

Current thresholds cannot be modified in-place. To change:

1. **De-register Old Vault** (soft deprecation, keep monitoring)
2. **Register New Vault** with updated threshold
3. **Migrate** vault operations to new registration
4. **Sunset** old vault after cutover

Process:
```bash
# Step 1: Register new vault with updated threshold
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source owner \
  --network testnet \
  -- register_vault \
  --vault <vault_address> \
  --owner <owner> \
  --evacuation <evacuation> \
  --token <token> \
  --threshold <new_threshold>

# Step 2: Migrate monitoring to new vault ID
# (Update off-chain indexer and monitoring bot)

# Step 3: Wait for confirmation all operations use new config

# Step 4: Note old vault ID as deprecated (optional cleanup)
```

### Adding/Removing Wardens

**Add Warden:**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- set_warden \
  --warden <new_warden_address> \
  --status true
```

**Remove Warden:**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- set_warden \
  --warden <revoked_warden_address> \
  --status false
```

Verify change:
- Confirm new warden can trip circuits
- Confirm revoked warden cannot trip circuits

### Contract Upgrade

**Prerequisites:**
- New WASM binary tested on testnet
- Security audit completed
- Stakeholder approval obtained
- Rollback plan documented

**Upgrade Steps:**

1. **Compile New WASM**
   ```bash
   cd contracts/sorosafe
   cargo build --target wasm32-unknown-unknown --release
   ```

2. **Calculate WASM Hash**
   ```bash
   sha256sum target/wasm32-unknown-unknown/release/sorosafe_circuit.wasm
   ```

3. **Execute Upgrade** (admin only)
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --source admin_hw_wallet \
     --network testnet \
     -- upgrade \
     --new_wasm_hash <hash_bytes_32>
   ```

4. **Verify Upgrade**
   - Check `Upgraded` event published
   - Verify all vault configurations persist
   - Run smoke tests on all vaults

5. **Monitor for Issues**
   - Watch event stream for anomalies
   - Keep rollback plan active for 24 hours

## Runbooks

### Runbook: Manual Emergency Stop

**When:** Exploit suspected, need to halt vault immediately
**Duration:** 5 minutes
**Risk:** Blocks all operations; coordinate with stakeholders

```bash
# 1. Identify target vault
VAULT="G..."

# 2. Confirm caller is authorized (owner or warden)
CALLER="<your_account>"

# 3. Execute manual trip
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source $CALLER \
  --network testnet \
  -- emergency_trip \
  --caller $CALLER \
  --vault $VAULT

# 4. Verify circuit is open
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- verify_state \
  --vault $VAULT \
  --current_balance <current_balance>
# Should return false

# 5. Notify stakeholders
echo "Circuit tripped for vault $VAULT by $CALLER at $(date)"
```

### Runbook: Execute Emergency Evacuation

**When:** Circuit is tripped and funds need to be moved to safety
**Duration:** 10 minutes
**Risk:** Transfers all vault funds; verify evacuation address beforehand

```bash
# 1. Verify circuit is open
VAULT="G..."
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- verify_state \
  --vault $VAULT \
  --current_balance <current_balance>
# Must return false

# 2. Execute evacuation (anyone can call, but only works if circuit is open)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source operator \
  --network testnet \
  -- execute_evacuation \
  --vault $VAULT

# 3. Verify evacuation happened
# Check event logs for Evacuated event with amount
# Verify evacuation address balance increased

# 4. Notify stakeholders
echo "Evacuation complete for vault $VAULT"
```

### Runbook: Warden Key Rotation

**When:** Routine rotation (quarterly) or suspected compromise
**Duration:** 30 minutes
**Risk:** Brief window where no active wardens if not overlapped

```bash
# 1. Add new warden
NEW_WARDEN="G..."
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- set_warden \
  --warden $NEW_WARDEN \
  --status true

# 2. Test new warden (optional, on testnet only)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source $NEW_WARDEN \
  --network testnet \
  -- emergency_trip \
  --caller $NEW_WARDEN \
  --vault <test_vault>

# 3. Remove old warden
OLD_WARDEN="G..."
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- set_warden \
  --warden $OLD_WARDEN \
  --status false

# 4. Verify
echo "Warden rotation complete:"
echo "  Removed: $OLD_WARDEN"
echo "  Added: $NEW_WARDEN"
```

## Metrics & KPIs

Track these metrics to assess circuit breaker effectiveness:

| Metric | Target | Frequency |
|--------|--------|-----------|
| Mean response time (trip to evacuation) | < 5 min | Event-based |
| False positive rate | < 5% | Monthly |
| Warden key rotation compliance | 100% | Quarterly |
| Monitoring uptime | > 99.9% | Daily |
| Event indexing lag | < 1 min | Continuous |

## Communication Plan

### Incident Notification

- **Severity Level:** Based on vault importance and funds at risk
- **Internal Alert:** Immediate
- **Stakeholder Notification:** Within 15 minutes
- **Public Status:** After analysis (if applicable)

### Status Page Template

```
[TIME] - Circuit Breaker Event

Vault: <vault_address>
Event Type: [Tripped_Auto | Tripped_Manual | Evacuated]
Amount: <tokens_if_evacuation>
Status: [Investigating | Contained | Resolved]

Actions Taken:
- ...

Next Steps:
- ...

Contact: <operations_email>
```

## References

- Full architecture: `docs/ARCHITECTURE.md`
- Security procedures: `docs/SECURITY.md`
- Deployment procedures: `docs/DEPLOYMENT.md`
