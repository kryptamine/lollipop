.DEFAULT_GOAL := help
.PHONY: help install dev dev-mock build format lint test check

MOCK_URL := http://127.0.0.1:2998/
MOCK_LOCKFILE := /tmp/lollipop-mock-lockfile
MOCK_TABLE := /tmp/lollipop-mock-benchmark-table.json

help: ## List the commands
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  %-10s %s\n", $$1, $$2}'

install: ## Install the view's dependencies
	pnpm install --frozen-lockfile

dev: ## Run the app against the real League client
	pnpm tauri dev

dev-mock: ## Run the app against scripts/mock-live-client.mjs, shown while this terminal is frontmost
	node scripts/mock-live-client.mjs $(MOCK_LOCKFILE) $(MOCK_TABLE) & trap "kill $$!" EXIT; LOLLIPOP_LIVE_URL=$(MOCK_URL) LOLLIPOP_LOCKFILE=$(MOCK_LOCKFILE) LOLLIPOP_BENCHMARK_TABLE=$(MOCK_TABLE) LOLLIPOP_GAME_BUNDLE=$$__CFBundleIdentifier pnpm tauri dev

build: ## Build the release bundle
	pnpm tauri build

format: ## Format the Rust and the view
	cargo fmt
	pnpm format

lint: ## Run every linter, Rust and view
	pnpm lint
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings
	scripts/check-boundaries.sh

test: ## Run the tests
	cargo test --workspace

check: lint test ## Run the linters and the tests, required before a push
	pnpm build
