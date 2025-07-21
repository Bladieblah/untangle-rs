.PHONY: help

help: ## Display the available options
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST)  | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

test: ## Run the tests
	poetry run pytest -vv

lint: ## Run linting
	ruff format
	ruff check --fix

build: ## Compile the rust code using maturin
	maturin develop

all: help
