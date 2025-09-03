.PHONY: help test lint

help: ## Display the available options
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST)  | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

build: ## Compile the rust code using maturin
	maturin develop

release: ## Compile the rust code using maturin
	maturin develop --release

test-rust:
	cargo test -p untanglers-core -- --nocapture

test-python: release ## Run the tests
	pytest -vv

test: test-rust test-python

benchmark: release ## Run a benchmark to compare the original python implementation with the optimised rust implementation
	cargo run -p untanglers-core --release --bin benchmark
	python -m tests.benchmark

lint: ## Run linting
	cargo fmt --all
	cargo clippy --fix --allow-dirty
	cargo clippy --all-targets --all-features -- -D warnings

	ruff format
	ruff check --fix

all: help
