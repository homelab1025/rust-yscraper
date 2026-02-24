# Suggested Commands

## Running the Project
- **Server**: `cargo run -p web_server` (requires `config.toml` in CWD).
- **API Client Gen**: `cargo run -p api_gen -- openapi.yaml`.
- **Frontend Dev**: `cd webapp && npm run dev`.

## Database
- **Postgres**: `docker compose up -d postgres`.
- **Migrations**: `docker compose up --no-deps liquibase`.

## Quality Control
- **Formatting**: `cargo fmt --all`.
- **Linting**: `cargo clippy --all-targets --all-features -D warnings`.
- **Testing**: `cargo test -p web_server`.
- **Coverage**: `cargo llvm-cov --html`.
