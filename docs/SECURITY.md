# SoroSafe Security Documentation

## Security Model

SoroSafe is designed with a security model emphasizing:
- **Explicit authentication** for all state changes
- **Role-based access control** (Owner, Warden, Admin)
- **Immutable event trails** for off-chain auditing
- **Fail-safe semantics** - circuit defaults to OPEN in case of anomalies

## Access Control

### Admin Role
- **Capability:** Initialize contract, upgrade WASM, manage wardens
- **Risk:** Single point of failure for contract operations
- **Mitigation:** Use hardware wallet or multi-sig account

### Vault Owner
- **Capability:** Register vault, trigger emergency stop, authorize evacuation
- **Risk:** Private key compromise exposes vault configuration
- **Mitigation:** Rotate keys regularly, use threshold signatures

### Warden Role
- **Capability:** Trigger emergency stop only (cannot access funds)
- **Risk:** Key compromise allows unauthorized circuit trips
- **Mitigation:** Limited scope reduces blast radius; distribute across multiple accounts

### Public/Monitoring Role
- **Capability:** Query vault state and execute evacuation
- **Risk:** DoS attacks on state queries (mitigation: RPC rate limiting)
- **Mitigation:** Execution requires circuit to be OPEN; state queries are read-only

## Threat Model

### Threat: Exploit drains vault rapidly

**Attack Vector:** Malicious contract or compromised vault logic
**Detection:** `verify_state()` compares `current_balance` against `last_balance + threshold`
**Response:** Circuit opens, blocking further operations; evacuation executes

**Assumptions:**
- Threshold is calibrated appropriately for operational patterns
- State verification is called frequently (e.g., every transaction)
- Monitoring is responsive

**Residual Risk:** If threshold is too high, significant loss occurs before trip.
**Mitigation:** Conservative thresholds, high-frequency state checks

---

### Threat: Unauthorized emergency stop

**Attack Vector:** Compromised warden key or social engineering
**Detection:** Emergency trip is protected by `require_auth()` and warden checks
**Response:** Audit logs show who triggered trip; can re-register vault

**Assumptions:**
- Warden keys are properly secured
- Audit logs are accessible and tamper-proof
- Operational procedures exist to respond to false positives

**Residual Risk:** Legitimate vault operations halted temporarily
**Mitigation:** Multi-warden governance, notification procedures

---

### Threat: Admin key compromise

**Attack Vector:** Private key exposure or infrastructure breach
**Detection:** Unauthorized WASM upgrade or warden configuration changes
**Response:** Community fork/mitigation if detected on mainnet

**Assumptions:**
- Admin key is protected with hardware wallet or multi-sig
- Upgrade events are monitored

**Residual Risk:** Complete loss of control over contract evolution
**Mitigation:** Use cold storage for admin; implement community governance (future)

---

### Threat: Token transfer fails silently

**Attack Vector:** Evacuation executed but tokens not transferred due to contract bug
**Detection:** Events logged but balances inconsistent
**Response:** Manual token sweep by vault operator

**Assumptions:**
- Token contract implements standard Soroban token interface
- Evacuation address is correct and not frozen

**Residual Risk:** Funds stranded in vault after circuit trip
**Mitigation:** Test token transfers on testnet; verify token contract audit

---

### Threat: Reentrancy during evacuation

**Attack Vector:** Token contract callback triggers nested state mutation
**Detection:** Soroban sandbox prevents direct reentrancy
**Response:** Not applicable - Soroban is non-reentrant by design

**Assumptions:**
- Soroban runtime enforces reentrancy protection

---

### Threat: Storage corruption or ledger state inconsistency

**Attack Vector:** Byzantine validator or ledger state divergence
**Detection:** Events diverge from queried state
**Response:** Off-chain monitoring detects discrepancies

**Assumptions:**
- Stellar consensus is honest (Byzantine agreement threshold)
- Off-chain indexing is independent and can cross-validate

**Residual Risk:** Transient state inconsistencies could cause false positives
**Mitigation:** Implement grace periods in monitoring logic; use consensus-based state queries

## Code Audit Checklist

- [ ] All state-mutating functions require `require_auth()`
- [ ] Error cases panic with descriptive `CircuitError` codes
- [ ] No unchecked arithmetic or overflow conditions (Rust safety)
- [ ] Token transfer amount and recipient are verified
- [ ] Threshold comparison logic is correct (`drain_amount > threshold`)
- [ ] TTL extension prevents premature storage expiry
- [ ] Events are emitted for all state transitions
- [ ] Test coverage includes happy paths and error cases
- [ ] No hardcoded addresses or magic numbers
- [ ] WASM binary size is within limits (~64KB is safe)

## Operational Security

### Key Management

**Admin Key:**
- Store in hardware wallet (Ledger, Trezor) or multi-sig
- Rotate periodically
- Never expose seed phrase

**Warden Keys:**
- Distribute across independent security zones
- Consider threshold signing (e.g., 2-of-3 multisig)
- Monitor for suspicious activity

**Vault Owner Keys:**
- Protected to same standard as admin
- Consider time-locked or emergency recovery options

### Monitoring

Deploy comprehensive monitoring for:

1. **Circuit State Changes**
   ```
   Event: Tripped_Auto → Threshold violation detected
   Action: Alert operators immediately; prepare evacuation
   
   Event: Tripped_Manual → Emergency stop triggered
   Action: Investigate cause; coordinate response
   ```

2. **Authorization Anomalies**
   ```
   Pattern: Repeated failed authorization attempts
   Action: Rate limit, investigate potential attacker
   ```

3. **Storage Changes**
   ```
   Event: Warden set/removed
   Action: Audit and approve
   ```

### Incident Response

**If circuit trips unexpectedly:**
1. Check `Tripped_Auto` event for vault address and balance
2. Verify threshold calibration is appropriate
3. Inspect vault contract for anomalies
4. If false positive, pause monitoring and re-calibrate

**If unauthorized trip detected:**
1. Collect event logs and transaction signatures
2. Identify compromised key
3. Rotate warden keys immediately
4. Register new vault with updated wardens

**If evacuation fails:**
1. Check token contract and evacuation address
2. Verify token balance is non-zero
3. Check allowance/spender configuration
4. Retry evacuation or perform manual sweep

### Testing on Testnet

Before mainnet deployment:

1. **Circuit trip under normal operation**
   - Verify threshold is neither too loose nor too strict
   - Confirm state queries reflect actual balances

2. **Warden authorization**
   - Confirm all designated wardens can trip circuit
   - Verify non-wardens are rejected

3. **Token transfer integration**
   - Evacuate small amounts first
   - Verify tokens arrive at evacuation address
   - Check token contract logs

4. **Multi-vault scenarios**
   - Test managing 3+ vaults concurrently
   - Verify vault states are independent
   - Confirm evacuation affects only target vault

5. **Stress and edge cases**
   - Rapid balance fluctuations
   - Boundary threshold values (0, i128::MAX)
   - Storage expiry and TTL extension

## Compliance Considerations

### Audit Trail
SoroSafe maintains comprehensive audit trails via Soroban events:
- Every vault registration is logged with owner and parameters
- Every trip (auto or manual) records time and trigger type
- Every evacuation records amounts transferred

These events are immutable on-chain and can be indexed for regulatory compliance.

### Regulatory Notes

SoroSafe itself is a neutral technology. Deployment context determines compliance obligations:

- **Custody:** If vault is customer funds, ensure evacuation address is compliant custody
- **Emergency Procedures:** Document and practice incident response
- **KYC/AML:** Apply to accounts authorized as owners and wardens
- **Disclosures:** Inform users of circuit breaker thresholds and emergency procedures

## References

- Soroban SDK Security: https://developers.stellar.org/docs/build/smart-contracts/security
- Soroban Audit Guide: https://github.com/stellar/soroban-examples
- Stellar Consensus Protocol: https://stellar.org/papers/stellar-consensus-protocol
