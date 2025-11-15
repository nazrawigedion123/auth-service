# ==============================================================================
# Diesel Migration Makefile
# 
# Prerequisites:
# 1. 'diesel_cli' must be installed: `cargo install diesel_cli --no-default-features --features postgres`
# 2. The 'auth-db' container must be running (docker compose up -d).
# 3. The AUTH_DB_PASSWORD environment variable must be set in your shell.
# ==============================================================================

# --- Variables ---

# Host and Port (use 'auth-db' for connections from inside Docker network, or 'localhost' if connecting locally)
DB_HOST ?= localhost
DB_PORT ?= 5432

# Database name and user from your docker-compose.yml
DB_NAME := my_auth_service_db
DB_USER := auth_user
DB_PASSWORD:= secret

# Construct the DATABASE_URL using variables (requires AUTH_DB_PASSWORD to be set externally)
DATABASE_URL := postgresql://$(DB_USER):$(DB_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)
export DATABASE_URL

# --- Targets ---

.PHONY: migration-new migrate

# Usage: make migration-new name=<your_migration_name>
# Example: make migration-new name=create_users_table
migration-new:
	@if [ -z "$(name)" ]; then \
		echo "Error: Migration name not provided. Use 'make migration-new name=<name>'"; \
		exit 1; \
	fi
	@echo "Creating new migration: $(name)..."
	diesel migration generate $(name)

# Runs all pending 'up' migrations against the database.
migrate:
	@echo "Running pending migrations on $(DB_NAME) at $(DB_HOST)..."
	diesel migration run

# Reverts the last applied migration. Use with caution.
migrate-revert:
	@echo "Reverting last migration..."
	diesel migration revert

# Shows the current status of all migrations (applied or pending).
migrate-status:
	@echo "Checking migration status..."
	diesel migration status

# Generates the 'schema.rs' file based on the current database state (e.g., after running 'migrate').
schema:
	@echo "Generating Rust schema.rs file..."
	diesel print-schema > src/schema.rs # NOTE: Assumes your schema file lives in src/schema.rs
