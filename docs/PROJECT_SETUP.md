# SoroSafe Project Setup Guide

## Initial Setup

### 1. Prerequisites

**Required:**
- Rust 1.70+ (https://rustup.rs/)
- Node.js 18+ (https://nodejs.org/)
- Git

**For Deployment:**
- Stellar CLI: `cargo install stellar-cli`
- A Stellar testnet account with XLM (get from https://friendbot.stellar.org/)

### 2. Verify Rust Installation

```bash
rustc --version
cargo --version
```

Expected output:
```
rustc 1.xx.x (...)
cargo 1.xx.x (...)
```

Add WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

### 3. Clone and Setup Repository

```bash
git clone <repository_url> sorosafe-circuit
cd sorosafe-circuit
```

### 4. Project Structure

```
sorosafe-circuit/
├── contracts/sorosafe/              # Smart contract (Rust)
│   ├── Cargo.toml                  # Rust dependencies
│   └── src/
│       ├── lib.rs                  # Main contract logic
│       ├── types.rs                # Data structures
│       └── test.rs                 # Unit tests
├── frontend/                        # Web dashboard (Next.js)
│   ├── package.json                # Node dependencies
│   ├── tsconfig.json               # TypeScript config
│   ├── next.config.js              # Next.js config
│   └── src/
│       ├── app/                    # Next.js pages
│       └── components/             # React components
├── scripts/
│   └── deploy.sh                   # Testnet deployment
├── docs/
│   ├── ARCHITECTURE.md             # Design documentation
│   ├── DEPLOYMENT.md               # Deployment procedures
│   ├── SECURITY.md                 # Security analysis
│   └── PROJECT_SETUP.md            # This file
├── Makefile                        # Build automation
└── README.md                       # Project overview
```

## Development Workflow

### Build Smart Contract

```bash
make contract-build
```

Or manually:
```bash
cd contracts/sorosafe
cargo build --target wasm32-unknown-unknown --release
```

Output: `contracts/sorosafe/target/wasm32-unknown-unknown/release/sorosafe_circuit.wasm`

### Run Contract Tests

```bash
make contract-test
```

Or manually:
```bash
cd contracts/sorosafe
cargo test
```

Expected: All tests pass ✓

### Build Frontend

```bash
make frontend-build
```

Or manually:
```bash
cd frontend
npm install
npm run build
```

### Run Frontend Dev Server

```bash
make frontend-dev
```

Visit `http://localhost:3000` to access the dashboard.

## Configuration

### Contract Network Settings

Edit `scripts/deploy.sh` to change network:

```bash
# For testnet (default)
NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org:443"
PASSPHRASE="Test SDF Network ; September 2015"

# For mainnet
NETWORK="mainnet"
RPC_URL="https://soroban-mainnet.stellar.org"
PASSPHRASE="Public Global Stellar Network ; September 2015"
```

### Frontend Network Settings

Create `frontend/src/config.ts`:

```typescript
export const CONFIG = {
  SOROBAN_RPC: process.env.NEXT_PUBLIC_SOROBAN_RPC || 'https://soroban-testnet.stellar.org',
  CONTRACT_ID: process.env.NEXT_PUBLIC_CONTRACT_ID || '',
  NETWORK: 'testnet',
  NETWORK_PASSPHRASE: 'Test SDF Network ; September 2015',
};
```

## Deployment

### Testnet Deployment

1. Ensure testnet account has XLM:
   ```bash
   stellar keys generate --network testnet deployer
   # Use Friendbot to fund: https://friendbot.stellar.org/
   ```

2. Deploy:
   ```bash
   make deploy
   ```

3. Save the contract ID printed at the end.

### Mainnet Deployment

See `docs/DEPLOYMENT.md` for detailed procedures including:
- Security considerations
- Hardware wallet integration
- Pre-deployment checklist
- Post-deployment verification

## Troubleshooting

### Rust/Cargo Issues

**"cargo: command not found"**
- Install Rust: https://rustup.rs/

**"failed to verify the checksum"**
```bash
cargo clean
rm -rf Cargo.lock
cargo build
```

**WASM target not found**
```bash
rustup target add wasm32-unknown-unknown
```

### Node.js/Frontend Issues

**"npm: command not found"**
- Install Node.js: https://nodejs.org/

**Port 3000 already in use**
```bash
make frontend-dev -- -p 3001
# or
cd frontend && npm run dev -- -p 3001
```

**Next.js build fails**
```bash
cd frontend
rm -rf node_modules .next package-lock.json
npm install
npm run build
```

### Deployment Issues

**"Contract not found on chain"**
- Verify contract ID is correct
- Check network (testnet vs mainnet)
- Verify RPC endpoint is accessible

**"Not authorized to deploy"**
- Ensure deployer account is funded
- Check account has correct network passphrase

**"WASM size too large"**
- This shouldn't happen (sorosafe is ~10KB)
- Try: `cargo build --target wasm32-unknown-unknown --release`

## Development Best Practices

### Before Committing

```bash
# Run all tests
make test

# Build everything
make build

# Check code formatting (Rust)
cd contracts/sorosafe && cargo fmt --check

# Lint frontend
cd frontend && npm run lint
```

### Code Style

**Rust:**
- Follow clippy recommendations: `cargo clippy`
- Use `cargo fmt` for formatting

**TypeScript/React:**
- Use ESLint configuration in `frontend/`
- Follow Next.js conventions

### Adding Dependencies

**Contract:**
```bash
cd contracts/sorosafe
cargo add <crate_name>
```

**Frontend:**
```bash
cd frontend
npm install <package_name>
```

## Performance Optimization

### Contract

Already optimized in `Cargo.toml`:
```toml
[profile.release]
opt-level = "z"      # Smallest binary size
overflow-checks = true
lto = true           # Link-time optimization
```

### Frontend

Build output is in `.next/static/` for CDN deployment.

## Monitoring & Debugging

### Contract Events

Query events from Soroban RPC:
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- query_events
```

### Frontend Debugging

Open browser DevTools (F12):
- Check Console for errors
- Inspect Network tab for RPC calls
- Use React DevTools (browser extension)

## CI/CD Setup (Optional)

### GitHub Actions Example

Create `.github/workflows/test.yml`:

```yaml
name: Test

on: [push, pull_request]

jobs:
  contract:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
      - run: cd contracts/sorosafe && cargo test
  
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18
      - run: cd frontend && npm ci && npm run build
```

## Support & Resources

- **Soroban Docs:** https://developers.stellar.org/docs/build/smart-contracts
- **Stellar SDK:** https://github.com/stellar/stellar-sdk
- **Next.js Docs:** https://nextjs.org/docs
- **Rust Book:** https://doc.rust-lang.org/book/
