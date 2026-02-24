# Shade Protocol - Development Makefile
# Comprehensive automation for building, testing, deploying, and managing Stellar smart contracts

# ============================================================================
# Configuration Variables
# ============================================================================

# Network configuration (testnet or futurenet)
NETWORK ?= testnet

# Admin address for contract initialization and management
ADMIN ?=

# Contract WASM paths
SHADE_WASM = target/wasm32-unknown-unknown/release/shade.wasm
ACCOUNT_WASM = target/wasm32-unknown-unknown/release/account.wasm

# Optimized WASM paths
SHADE_WASM_OPT = target/wasm32-unknown-unknown/release/shade.optimized.wasm
ACCOUNT_WASM_OPT = target/wasm32-unknown-unknown/release/account.optimized.wasm

# Contract ID storage directory
STELLAR_DIR = .stellar

# ============================================================================
# Phony Targets
# ============================================================================

.PHONY: help build test optimize clean fmt lint \
	deploy-shade deploy-account deploy-all \
	init-shade \
	pause-shade unpause-shade upgrade-shade \
	check-admin check-network

# ============================================================================
# Default Target
# ============================================================================

.DEFAULT_GOAL := help

# ============================================================================
# Help Target
# ============================================================================

help: ## Show this help message
	@echo "Shade Protocol - Development Makefile"
	@echo ""
	@echo "Usage: make [target] [VARIABLE=value]"
	@echo ""
	@echo "Configuration Variables:"
	@echo "  NETWORK=testnet|futurenet  Network to deploy to (default: testnet)"
	@echo "  ADMIN=<address>            Admin address for contract operations"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  %-25s %s\n", $$1, $$2}'

# ============================================================================
# Common Development Targets
# ============================================================================

build: ## Build all contracts
	@echo "Building contracts..."
	@cargo build --target wasm32-unknown-unknown --release
	@ls -lh $(SHADE_WASM) $(ACCOUNT_WASM)

test: ## Run all tests
	@echo "Running tests..."
	@cargo test --all

optimize: build ## Optimize WASM binaries
	@echo "Optimizing contracts..."
	@stellar contract optimize --wasm $(SHADE_WASM)
	@stellar contract optimize --wasm $(ACCOUNT_WASM)
	@ls -lh $(SHADE_WASM_OPT) $(ACCOUNT_WASM_OPT)

fmt: ## Format code
	@echo "Formatting code..."
	@cargo fmt --all

lint: ## Run clippy linter
	@echo "Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf $(STELLAR_DIR)

# ============================================================================
# Deployment Targets
# ============================================================================

check-network:
	@if [ -z "$(NETWORK)" ]; then \
		echo "Error: NETWORK is not set. Use NETWORK=testnet or NETWORK=futurenet"; \
		exit 1; \
	fi

check-admin:
	@if [ -z "$(ADMIN)" ]; then \
		echo "Error: ADMIN address is required. Set ADMIN=<address>"; \
		exit 1; \
	fi

$(STELLAR_DIR):
	@mkdir -p $(STELLAR_DIR)

deploy-shade: optimize check-network $(STELLAR_DIR) ## Deploy Shade contract
	@echo "Deploying Shade contract to $(NETWORK)..."
	@stellar contract deploy \
		--wasm $(SHADE_WASM_OPT) \
		--network $(NETWORK) \
		--source-account default \
		> $(STELLAR_DIR)/shade_contract_id.txt
	@echo "Shade contract deployed!"
	@echo "Contract ID: $$(cat $(STELLAR_DIR)/shade_contract_id.txt)"

deploy-account: optimize check-network $(STELLAR_DIR) ## Deploy Account contract
	@echo "Deploying Account contract to $(NETWORK)..."
	@stellar contract deploy \
		--wasm $(ACCOUNT_WASM_OPT) \
		--network $(NETWORK) \
		--source-account default \
		> $(STELLAR_DIR)/account_contract_id.txt
	@echo "Account contract deployed!"
	@echo "Contract ID: $$(cat $(STELLAR_DIR)/account_contract_id.txt)"

deploy-all: deploy-shade deploy-account ## Deploy all contracts

# ============================================================================
# Initialization Targets
# ============================================================================

init-shade: check-admin check-network ## Initialize Shade contract with admin
	@if [ ! -f $(STELLAR_DIR)/shade_contract_id.txt ]; then \
		echo "Error: Shade contract not deployed. Run 'make deploy-shade' first."; \
		exit 1; \
	fi
	@echo "Initializing Shade contract..."
	@stellar contract invoke \
		--id $$(cat $(STELLAR_DIR)/shade_contract_id.txt) \
		--network $(NETWORK) \
		--source-account default \
		-- \
		initialize \
		--admin $(ADMIN)
	@echo "Shade contract initialized with admin: $(ADMIN)"

# ============================================================================
# Contract Management Targets
# ============================================================================

pause-shade: check-admin check-network ## Pause Shade contract
	@if [ ! -f $(STELLAR_DIR)/shade_contract_id.txt ]; then \
		echo "Error: Shade contract not deployed."; \
		exit 1; \
	fi
	@echo "Pausing Shade contract..."
	@stellar contract invoke \
		--id $$(cat $(STELLAR_DIR)/shade_contract_id.txt) \
		--network $(NETWORK) \
		--source-account default \
		-- \
		pause \
		--admin $(ADMIN)
	@echo "Shade contract paused"

unpause-shade: check-admin check-network ## Unpause Shade contract
	@if [ ! -f $(STELLAR_DIR)/shade_contract_id.txt ]; then \
		echo "Error: Shade contract not deployed."; \
		exit 1; \
	fi
	@echo "Unpausing Shade contract..."
	@stellar contract invoke \
		--id $$(cat $(STELLAR_DIR)/shade_contract_id.txt) \
		--network $(NETWORK) \
		--source-account default \
		-- \
		unpause \
		--admin $(ADMIN)
	@echo "Shade contract unpaused"

upgrade-shade: optimize check-network ## Upgrade Shade contract (requires new WASM)
	@if [ ! -f $(STELLAR_DIR)/shade_contract_id.txt ]; then \
		echo "Error: Shade contract not deployed."; \
		exit 1; \
	fi
	@echo "Installing new WASM..."
	@stellar contract install \
		--wasm $(SHADE_WASM_OPT) \
		--network $(NETWORK) \
		--source-account default \
		> $(STELLAR_DIR)/shade_new_wasm_hash.txt
	@echo "New WASM hash: $$(cat $(STELLAR_DIR)/shade_new_wasm_hash.txt)"
	@echo "Note: Call upgrade function with this hash to complete upgrade"

# ============================================================================
# Query Targets
# ============================================================================

get-admin-shade: check-network ## Get Shade contract admin
	@if [ ! -f $(STELLAR_DIR)/shade_contract_id.txt ]; then \
		echo "Error: Shade contract not deployed."; \
		exit 1; \
	fi
	@stellar contract invoke \
		--id $$(cat $(STELLAR_DIR)/shade_contract_id.txt) \
		--network $(NETWORK) \
		-- \
		get_admin

is-paused-shade: check-network ## Check if Shade contract is paused
	@if [ ! -f $(STELLAR_DIR)/shade_contract_id.txt ]; then \
		echo "Error: Shade contract not deployed."; \
		exit 1; \
	fi
	@stellar contract invoke \
		--id $$(cat $(STELLAR_DIR)/shade_contract_id.txt) \
		--network $(NETWORK) \
		-- \
		is_paused

# ============================================================================
# Development Workflow Targets
# ============================================================================

dev-setup: build test ## Initial development setup
	@echo "Development setup complete!"

deploy-testnet: NETWORK=testnet
deploy-testnet: deploy-all ## Deploy all contracts to testnet

deploy-futurenet: NETWORK=futurenet
deploy-futurenet: deploy-all ## Deploy all contracts to futurenet

ci: lint test build ## CI pipeline (lint, test, build)
	@echo "CI pipeline complete!"
