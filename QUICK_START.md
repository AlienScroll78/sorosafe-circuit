# SoroSafe Quick Start

Get started in 15 minutes.

## 1. Clone & Setup (2 minutes)

```bash
cd sorosafe-circuit

# Verify Rust is installed
rustc --version

# Add WASM target
rustup target add wasm32-unknown-unknown
```

## 2. Build Contract (3 minutes)

```bash
cd contracts/sorosafe
cargo build --target wasm32-unknown-unknown --release
cargo test

# Expected output: "test result: ok"
```

## 3. Deploy to Testnet (5 minutes)

```bash
cd ../..

# Get testnet XLM from https://friendbot.stellar.org/

chmod +x scripts/deploy.sh
./scripts/deploy.sh

# Save the Contract ID printed at the end
```

## 4. Register a Vault (3 minutes)

```bash
# Set variables
CONTRACT_ID="CABC..."  # From step 3
VAULT="GVAULT..."
OWNER="GOWNER..."
EVAC="GEVAC..."
TOKEN="CTOKEN..."

# Register
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $OWNER \
  --network testnet \
  -- register_vault \
  --vault $VAULT \
  --owner $OWNER \
  --evacuation $EVAC \
  --token $TOKEN \
  --threshold 5000
```

## 5. Test Monitoring (2 minutes)

```bash
# Query healthy state
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- verify_state \
  --vault $VAULT \
  --current_balance 100000
# Returns: true (healthy)

# Simulate balance drop exceeding threshold
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- verify_state \
  --vault $VAULT \
  --current_balance 94000
# Returns: false (circuit tripped!)
```

## Common Commands

### Verify Setup
```bash
make verify-setup
```

### Build Everything
```bash
make build
```

### Run Tests
```bash
make contract-test
```

### Start Frontend Dev
```bash
make frontend-dev
# Visit http://localhost:3000
```

### Clean Build Artifacts
```bash
make clean
```

## File Structure Quick Reference

```
sorosafe-circuit/
├── contracts/sorosafe/      # Smart contract
│   └── src/
│       ├── lib.rs          # Main logic
│       ├── types.rs        # Data structures
│       └── test.rs         # Tests
├── frontend/                # Next.js dashboard
├── docs/                    # Full documentation
│   ├── ARCHITECTURE.md      # Design
│   ├── DEPLOYMENT.md        # Deploy guide
│   ├── OPERATIONS.md        # Run procedures
│   ├── TESTING.md           # Test guide
│   ├── SECURITY.md          # Security model
│   └── API_REFERENCE.md     # Full API
├── scripts/
│   └── deploy.sh            # Testnet deploy
└── README.md                # Project overview
```

## Next Steps

1. **Read Architecture:** `docs/ARCHITECTURE.md`
2. **Run Tests:** `make contract-test`
3. **Deploy to Testnet:** `scripts/deploy.sh`
4. **Test Monitoring:** Manual state queries
5. **Run Frontend:** `make frontend-dev`
6. **Review Operations:** `docs/OPERATIONS.md`
7. **Plan Mainnet:** `docs/DEPLOYMENT.md`

## Troubleshooting

**"cargo: command not found"**
→ Install Rust: https://rustup.rs/

**"stellar: command not found"**
→ Install Stellar CLI: `cargo install stellar-cli`

**Tests fail**
→ Run `cargo clean` then `cargo test` again

**Deployment fails**
→ Check your testnet account has XLM (use Friendbot)

## Key Concepts

| Term | Meaning |
|------|---------|
| **Circuit** | The breaker mechanism; CLOSED (normal) or OPEN (tripped) |
| **Vault** | The account/contract being monitored |
| **Threshold** | Maximum allowed balance drop before auto-trip |
| **Warden** | Account authorized to manually trip circuit |
| **Evacuation** | Safe address where funds transfer when circuit opens |

## Contact & Support

- **Docs:** See `docs/` directory
- **Issues:** Review `docs/SECURITY.md` for security concerns
- **Questions:** Check test files for usage examples
