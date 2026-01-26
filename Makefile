# Makefile for sobriquet

.PHONY: help
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the project
	cargo build

.PHONY: build-release
build-release: ## Build release version
	cargo build --release

.PHONY: test
test: ## Run tests
	cargo test

.PHONY: test-all
test-all: ## Run all tests including integration tests
	cargo test --all-features

.PHONY: lint
lint: ## Run clippy
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: fmt
fmt: ## Format code
	cargo fmt

.PHONY: fmt-check
fmt-check: ## Check code formatting
	cargo fmt -- --check

.PHONY: check
check: fmt-check lint test ## Run all checks (format, lint, test)

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

.PHONY: install
install: ## Install sobriquet locally
	cargo install --path .

.PHONY: release
release: ## Prepare a new release (usage: make release VERSION=0.3.0)
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make release VERSION=0.3.0"; \
		exit 1; \
	fi
	@./scripts/bump-version.sh $(VERSION)

.PHONY: docs
docs: ## Build and open documentation
	cargo doc --open --no-deps

.PHONY: bench
bench: ## Run benchmarks
	cargo bench

.PHONY: coverage
coverage: ## Generate code coverage report (requires cargo-tarpaulin)
	cargo tarpaulin --out Html --output-dir coverage

.PHONY: audit
audit: ## Run security audit
	cargo audit

.PHONY: update
update: ## Update dependencies
	cargo update

.PHONY: watch
watch: ## Watch for changes and run tests (requires cargo-watch)
	cargo watch -x test

.DEFAULT_GOAL := help
