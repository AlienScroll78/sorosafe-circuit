.PHONY: help build test deploy clean contract-build contract-test frontend-build frontend-dev

help:
	@echo "SoroSafe Circuit - Build & Deployment"
	@echo ""
	@echo "Contract Targets:"
	@echo "  make contract-build     Build WASM contract"
	@echo "  make contract-test      Run contract unit tests"
	@echo "  make contract-clean     Clean build artifacts"
	@echo ""
	@echo "Frontend Targets:"
	@echo "  make frontend-build     Build Next.js production build"
	@echo "  make frontend-dev       Start development server (http://localhost:3000)"
	@echo ""
	@echo "Full Targets:"
	@echo "  make build              Build contract and frontend"
	@echo "  make test               Run all tests"
	@echo "  make deploy             Deploy contract to testnet (requires env setup)"
	@echo "  make clean              Clean all artifacts"

contract-build:
	@echo "Building Soroban WASM contract..."
	cd contracts/sorosafe && \
	rustup target add wasm32-unknown-unknown && \
	cargo build --target wasm32-unknown-unknown --release

contract-test:
	@echo "Running contract tests..."
	cd contracts/sorosafe && cargo test

contract-clean:
	@echo "Cleaning contract build artifacts..."
	cd contracts/sorosafe && cargo clean

frontend-build:
	@echo "Building Next.js frontend..."
	cd frontend && npm ci && npm run build

frontend-dev:
	@echo "Starting frontend development server..."
	@echo "Visit http://localhost:3000"
	cd frontend && npm install && npm run dev

build: contract-build frontend-build
	@echo "✓ Build complete"

test: contract-test
	@echo "✓ Tests complete"

deploy:
	@echo "Deploying to Soroban testnet..."
	chmod +x scripts/deploy.sh
	./scripts/deploy.sh

clean: contract-clean
	@echo "Cleaning all artifacts..."
	cd frontend && rm -rf node_modules .next && npm cache clean --force || true
	@echo "✓ Clean complete"

verify-setup:
	@echo "Verifying development environment..."
	@which cargo > /dev/null || (echo "Error: Rust/cargo not found"; exit 1)
	@which node > /dev/null || (echo "Error: Node.js not found"; exit 1)
	@which npm > /dev/null || (echo "Error: npm not found"; exit 1)
	@which stellar > /dev/null || (echo "Warning: Stellar CLI not installed (needed for deployment)")
	@echo "✓ Development environment verified"
