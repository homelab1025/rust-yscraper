# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Code Handling
- use Serena as an MCP so to get accurate information about the codebase (using LSP) whenever reading or writing code.

## Commands

```bash
# Run the server (from repo root, requires config.toml and a running Postgres)
cargo run -p web_server

# Run all tests (no DB required — unit tests use mocks)
cargo test -p web_server

# Run a single test by name substring
cargo test -p web_server <test_name>
# e.g. cargo test -p web_server failing_task

# Coverage report (requires cargo-llvm-cov)
cargo llvm-cov --html

# Generate OpenAPI YAML
cargo run -p api_gen -- openapi.yaml

# Spin up Postgres + run Liquibase migrations
docker compose up -d postgres
docker compose up --no-deps liquibase

# Frontend dev server
cd webapp && npm install && npm run dev   # http://localhost:5173
```

## Configuration

The server reads `config.toml` from the **current working directory** at startup. A dev copy lives at
`conf/config.toml`. Copy it to the repo root before running locally. All keys can be overridden with environment
variables prefixed `YSCR_` (e.g. `YSCR_DB_PASSWORD`).

## Architecture

The workspace has two crates: `web_server` (the main service) and `api_gen` (dumps the OpenAPI YAML by importing
`web_server::api::ApiDoc`).

### Key architectural patterns

**Repository trait pattern.** All DB access goes through entity dedicated traits. The single concrete implementation is
`PgCommentsRepository`, which implements all traits. `CombinedRepository` is a blanket supertrait. Tests mock the traits
directly — no DB needed.

**Task deduplication queue.** `TaskDedupQueue<T>` maintains a set of `T` tasks and an `mpsc` channel. `schedule()`
acquires the lock, rejects the task if already present (returns `Ok(false)`), otherwise inserts and `try_send`s it. A
single worker loop spawned in `new()` processes tasks sequentially, calling `execute()` and removing the task from the
set after completion (success or error). Task identity is defined by the `Hash + Eq` impl on `T` — for `ScrapeTask` this
is `(url, url_id)`.

**Axum state extraction.** `AppState` is the top-level state. Handler-specific substates (e.g. `LinksAppState`)
implement `FromRef<AppState>` so handlers can extract only what they need.

### Database schema

- database schema is managed using Liquibase(`db/changelog/`)
- the HN item ID is used directly as the primary key for both `urls.id` and `comments.id`.
- when schema is changed, you can't just change the DDL sql files in the `db/changelog/` folder, but have to add a new change to the `changelog-master.yaml` file and the corresponding sql file for the change.

### Scraping

- on scraping, comments are upserted in batches of 10.
