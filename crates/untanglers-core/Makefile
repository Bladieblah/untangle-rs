.PHONY: help test lint

help: ## Display the available options
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST)  | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

test: ## Run tests
	cargo test -- --nocapture

lint: ## Run linting
	cargo fmt --all
	cargo clippy --fix --allow-dirty
	cargo clippy --all-targets --all-features -- -D warnings

all: help
