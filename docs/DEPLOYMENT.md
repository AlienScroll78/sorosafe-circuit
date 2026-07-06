# SoroSafe Deployment Guide

## Prerequisites

- Rust 1.70+
- Stellar CLI: `cargo install stellar-cli`
- Node.js 18+ (for frontend)
- A funded testnet account (get XLM from https://friendbot.stellar.org/)

## Testnet Deployment

### 1. Build the Contract

```bash
cd contracts/sorosafe
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
cd ../..
```

Output: `contracts/sorosafe/target/wasm32-unknown-unknown/release/sorosafe_circuit.wasm`

### 2. Deploy to Testnet

```bash
chmod +x scripts/deploy.sh
./scripts/deploy.sh
```

The script will:
1. Build the WASM contract
2. Configure Stellar CLI for testnet
3. Generate/use an admin deployer key
4. Deploy the contract to testnet
5. Initialize the admin account

**Save the contract ID** - you'll need it for vault registration and operations.

### 3. Register a Vault

After deployment, register a vault:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source vault_owner_account \
  --network testnet \
  -- register_vault \
  --vault <vault_address> \
  --owner <owner_address> \
  --evacuation <emergency_address> \
  --token <token_contract_address> \
  --threshold 500
```

Parameters:
- `vault`: The vault contract address to monitor
- `owner`: Principal account (must sign this tx)
- `evacuation`: Safe address for emergency transfers
- `token`: Soroban token contract address
- `threshold`: Max allowed drain in atomic units (e.g., 500)

### 4. Authorize Token Transfers

The circuit breaker contract must be authorized to transfer tokens from the vault.

If the vault uses standard Soroban token interface:

```bash
stellar contract invoke \
  --id <TOKEN_CONTRACT_ID> \
  --source vault_account \
  --network testnet \
  -- approve \
  --from <vault_address> \
  --spender <SOROSAFE_CONTRACT_ID> \
  --amount <large_amount> \
  --expiration_ledger 1000000
```

### 5. Designate Wardens

Authorize accounts as wardens (optional, only admin):

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source admin_deployer \
  --network testnet \
  -- set_warden \
  --warden <warden_address> \
  --status true
```

## Monitoring Operations

### Query Vault State

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source monitoring_account \
  --network testnet \
  -- verify_state \
  --vault <vault_address> \
  --current_balance <balance>
```

Returns `true` if vault is healthy, `false` if circuit is tripped.

### Manual Emergency Trip

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source owner_or_warden_account \
  --network testnet \
  -- emergency_trip \
  --caller <caller_address> \
  --vault <vault_address>
```

### Execute Evacuation

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source any_account \
  --network testnet \
  -- execute_evacuation \
  --vault <vault_address>
```

## Frontend Setup

### 1. Install Dependencies

```bash
cd frontend
npm install
```

### 2. Configure Network

Update `frontend/src/config.ts` (create if needed):

```typescript
export const CONFIG = {
  SOROBAN_RPC: 'https://soroban-testnet.stellar.org',
  CONTRACT_ID: '<your_contract_id>',
  NETWORK: 'testnet',
  NETWORK_PASSPHRASE: 'Test SDF Network ; September 2015',
};
```

### 3. Run Development Server

```bash
npm run dev
```

Visit `http://localhost:3000`

### 4. Build for Production

```bash
npm run build
npm start
```

## Mainnet Deployment

### Pre-Deployment Checklist

- [ ] Contract tested on testnet with real scenarios
- [ ] All thresholds and evacuation addresses verified
- [ ] Warden accounts identified and secured
- [ ] Mainnet admin key in secure storage (hardware wallet recommended)
- [ ] Monitoring infrastructure deployed
- [ ] Emergency response procedures documented
- [ ] Security audit completed (recommended)

### Mainnet Deployment Steps

1. **Build and Verify**
   ```bash
   cd contracts/sorosafe
   cargo build --target wasm32-unknown-unknown --release
   # Verify the WASM hash matches your audit
   sha256sum target/wasm32-unknown-unknown/release/sorosafe_circuit.wasm
   ```

2. **Configure for Mainnet**
   ```bash
   stellar network add \
     --rpc-url "https://soroban-mainnet.stellar.org" \
     --network-passphrase "Public Global Stellar Network ; September 2015" \
     mainnet
   ```

3. **Deploy with Hardware Wallet**
   ```bash
   stellar contract deploy \
     --wasm contracts/sorosafe/target/wasm32-unknown-unknown/release/sorosafe_circuit.wasm \
     --source mainnet_admin_hw_wallet \
     --network mainnet
   ```

4. **Initialize and Verify**
   ```bash
   stellar contract invoke \
     --id <MAINNET_CONTRACT_ID> \
     --source mainnet_admin_hw_wallet \
     --network mainnet \
     -- init \
     --admin <admin_address>
   ```

5. **Document and Announce**
   - Publish contract ID and network info
   - Set up monitoring and alerts
   - Notify vault operators

## Contract Upgrade

To upgrade the contract to a new WASM:

1. Build and audit the new WASM
2. Compute the WASM hash:
   ```bash
   sha256sum target/wasm32-unknown-unknown/release/sorosafe_circuit.wasm
   ```
3. Call the upgrade function:
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --source admin \
     --network testnet \
     -- upgrade \
     --new_wasm_hash <hash_bytes_32>
   ```

**Note:** The contract emits an `Upgraded` event and preserves all vault configurations.

## Troubleshooting

### "Contract not found"
- Verify contract ID is correct
- Check network selection (testnet vs mainnet)
- Ensure RPC endpoint is accessible

### "Not authorized"
- Ensure the source account signs with correct keys
- Check that caller has required permissions (owner/warden/admin)

### "Vault not registered"
- Call `register_vault` first
- Verify vault address is correct

### "Circuit already open"
- Circuit cannot be reopened after tripping
- Register a new vault to reset monitoring

## Monitoring and Alerts

Set up event listeners for:
- `Tripped_Auto` - Threshold violation detected
- `Tripped_Manual` - Emergency stop triggered
- `Evacuated` - Assets transferred to safety

Example Soroban indexer query:
```sql
SELECT * FROM events 
WHERE contract_id = '<CONTRACT_ID>'
AND event_type IN ('Tripped_Auto', 'Tripped_Manual', 'Evacuated')
ORDER BY ledger_sequence DESC
```

## Support

For issues or questions:
1. Review `docs/ARCHITECTURE.md` for design details
2. Check test cases in `contracts/sorosafe/src/test.rs`
3. Review Soroban SDK docs: https://developers.stellar.org/docs/build/smart-contracts
