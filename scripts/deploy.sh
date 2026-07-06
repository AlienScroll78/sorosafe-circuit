#!/bin/bash
set -e

echo "=== 1. Building Contract ==="
cd contracts/sorosafe
cargo build --target wasm32-unknown-unknown --release
cd ../..

echo ""
echo "=== 2. Configuring Network ==="
stellar network add \
  --rpc-url "https://soroban-testnet.stellar.org:443" \
  --network-passphrase "Test SDF Network ; September 2015" \
  testnet || true

stellar keys generate --network testnet admin_deployer || true

echo ""
echo "=== 3. Deploying to Testnet ==="
CONTRACT_ID=$(stellar contract deploy \
  --wasm contracts/sorosafe/target/wasm32-unknown-unknown/release/sorosafe_circuit.wasm \
  --source admin_deployer \
  --network testnet)

echo "✓ Contract deployed successfully!"
echo "  Contract ID: $CONTRACT_ID"

echo ""
echo "=== 4. Initializing Admin ==="
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source admin_deployer \
  --network testnet \
  -- init \
  --admin admin_deployer

echo "✓ Admin initialized"

echo ""
echo "=== Deployment Complete ==="
echo "Save this Contract ID for future operations:"
echo "  $CONTRACT_ID"
