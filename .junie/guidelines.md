# Rust Guidelines for this project

These conventions tailor general Rust best practices to this repository. They complement the root `Guidelines.md` by focusing on Rust specifics: coding conventions, testing (unit and integration), and code organization.

## 1) Coding conventions

- Formatting
  - Run `cargo fmt --all` before committing. Keep `rustfmt` defaults; if you need to deviate, justify it in a comment and a `rustfmt.toml`.
- Linting
  - Run `cargo clippy --all-targets --all-features -D warnings` locally. Prefer fixing over `#[allow(...)]`. If an allow is justified, scope it to the smallest block and add a short comment.
- Naming & style
  - Follow Rust’s conventional casing: `snake_case` for functions/variables/modules, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants, and `lowercase` crate names.
  - Prefer explicit module `pub` visibility; don’t export internal helpers by default. Use `pub(crate)` when sharing within the crate.
- Errors & results
  - Return `Result<T, E>` over panics. Use `anyhow` for application-layer error aggregation and `thiserror` for library-style typed errors. If introducing either, add them to `Cargo.toml` with minimal features.
  - Avoid `unwrap()`/`expect()` in non-test code. Use `?` and map errors with context via `anyhow::Context` or meaningful error variants.
- Logging
  - Use `log` macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`). Keep logs structured and actionable. Avoid logging secrets.
- Concurrency & async
  - Use `tokio` primitives. Prefer `async fn` and `.await` over manual threads. Bound concurrency with `FuturesUnordered` or `Semaphore` rather than spawning unbounded tasks.
  - Avoid blocking calls on async threads (e.g., file I/O or CPU-heavy work) unless wrapped in `tokio::task::spawn_blocking`.
- Data access (sqlx / SQLite)
  - Use a single `SqlitePool` passed via dependency injection instead of globals. Prefer prepared statements with `.bind()` and avoid string concatenation.
  - Migrations should be applied deterministically; prefer `sqlx::migrate!()` for production code. For this project’s simplicity, if using ad-hoc `CREATE TABLE IF NOT EXISTS`, ensure idempotency.
- HTTP & scraping
  - Prefer a shared `reqwest::Client` with timeouts. Separate fetching from parsing (HTML selection via `scraper`).
- Documentation
  - Add `///` rustdoc comments to public items explaining invariants and examples. Use `cargo doc --no-deps` to check.

## 2) Code organization

- Crate layout
  - Keep the binary entry point minimal in `src/main.rs`. Move logic into modules under `src/`.
  - Typical structure for this project:
    - `src/main.rs` — CLI/startup, config, DI wiring, logging setup.
    - `src/scrape.rs` — networking and HTML parsing. Split into `fetch` (HTTP) and `parse` (DOM parsing) helpers if it grows.
    - `src/db.rs` — database pool init and queries (CRUD). Keep SQL in one place. Return domain types.
    - `src/model.rs` — domain structs (e.g., `CommentRecord`) with serde derives if needed.
    - `src/config.rs` — configuration loading (from `config` crate) with defaults.
  - If modules are small, keeping `scrape.rs` and DB helpers as private modules is fine; promote to separate files as they grow.
- Boundaries and dependencies
  - Separate concerns:
    - Service layer: orchestrates scrape, transform, persist.
    - Data layer: sqlx queries; no business logic.
    - IO layer: HTTP client setup and calls.
  - Pass dependencies explicitly as parameters (e.g., `&SqlitePool`, `&Client`). Avoid singletons.
- Batching and utilities
  - Keep general-purpose helpers (like `create_batches`) in a `util.rs` module if reused. Add tests and rustdoc examples.

## 3) Testing strategy

General principles
- Tests must be deterministic, isolated, and fast. Avoid network and disk where possible. Prefer pure functions and small units.
- Use descriptive test names and arrange-act-assert structure with clear comments when helpful.

Unit tests (colocated)
- Place unit tests in the same file within a `#[cfg(test)] mod tests { ... }` module.
- Use pure functions for parsing and transformation to make unit tests trivial.
- Example patterns used in this repo already:
  - Testing `create_batches` behavior for edge cases and typical cases.

Async tests
- Use `#[tokio::test(flavor = "current_thread")]` for async unit tests. Avoid multi-threaded test runtime unless necessary.

HTTP mocking
- Do not hit real network in tests. Use one of:
  - `wiremock` crate (powerful) or `httpmock` for a lightweight alternative.
  - For simple parsing tests, store minimal HTML fixtures in `tests/fixtures/` and call parsing functions directly, bypassing HTTP.

Database testing (SQLite)
- Prefer in-memory databases for unit/integration tests: `sqlite::memory:` or `SqlitePoolOptions::new().connect("sqlite::memory:")` (with `?cache=shared` if multiple connections are required).
- Apply schema setup per test. Keep schema idempotent.
- Ensure each test cleans up after itself; since memory DBs are ephemeral, dropping the pool is sufficient.

Integration tests
- Place black-box tests in `tests/` directory. Each file is a separate crate compiled against the binary/library.
- Cover end-to-end flows without external side effects:
  - Mock HTTP responses for a Hacker News item page.
  - Initialize an in-memory SQLite, run the flow that inserts comments, then assert counts/records.
- If the binary remains monolithic, expose a small public API (e.g., `run_once(config: &Config, pool: &SqlitePool)`) to drive from tests.

Property-based tests (optional)
- For data transformations (e.g., tag extraction), consider `proptest` with bounded sizes and timeouts.

Golden tests (optional)
- For HTML parsing selectors, store small, curated HTML snippets as fixtures and compare structured outputs. Keep fixtures minimal and documented.

## 4) Tooling and commands

- Format
  - `cargo fmt --all`
- Lint
  - `cargo clippy --all-targets --all-features -D warnings`
- Test
  - Unit tests: `cargo test --lib -- --nocapture`
  - All tests (incl. integration): `cargo test --all -- --nocapture`

## 5) Performance and reliability tips

- Bound parallelism on scraping and insertion to avoid overloading remote or local DB. Use batching (as done) with a configurable size.
- Use prepared statements and transactions for bulk inserts when speed matters.
- Add timeouts and retries for HTTP with jittered backoff in the future if networking becomes flaky.

## 6) Commit practices (project-specific)

- Small, focused commits with functional messages. If a commit is purely technical (e.g., refactor/format), include a brief rationale.
- Per repository workflow: prepare the commit message and ask for approval before committing.

---

Appendix: Suggested crates for testing (add only when needed)
- `dev-dependencies` examples:
  - `wiremock = "0.6"`
  - `httpmock = "0.7"`
  - `proptest = "1"`

These guidelines are living; update them when the architecture or constraints evolve.