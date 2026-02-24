# Coding Conventions

## Naming & Style
- `snake_case` for functions/variables/modules.
- `CamelCase` for types/traits.
- `SCREAMING_SNAKE_CASE` for constants.
- `lowercase` crate names.
- Prefer `pub(crate)` for sharing within crates; avoid unnecessary `pub`.

## Error Management
- Avoid `unwrap()`/`expect()`.
- Use `?` and return `Result<T, E>`.
- Only `thiserror` is acceptable; do NOT use `anyhow`.

## Async & Concurrency
- Use `tokio` primitives.
- Avoid blocking calls on async threads (use `spawn_blocking` if necessary).
- Bound concurrency (e.g., `FuturesUnordered`).
