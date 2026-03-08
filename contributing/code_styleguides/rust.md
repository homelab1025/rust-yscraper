# Effective Rust Style Guide

This document summarizes key rules and best practices for writing idiomatic, safe, and maintainable Rust code. It is derived from [Effective Rust](https://effective-rust.com/title-page.html) by David Drysdale (35 Specific Ways to Improve Your Rust Code) and tailored to this project's conventions.

---

## 1. Type System

### 1.1 Express Data Structures with Types (Item 1)
- **Encode meaning into the type system.** Don't represent distinct domain concepts with the same primitive type (e.g., don't use `f64` for both pounds-force-seconds and newton-seconds).
- **Prefer enums over booleans** for function arguments with more than two semantic choices. This improves readability and type safety at call sites.
  ```rust
  // ❌ Bad
  print_page(/* both_sides= */ true, /* color= */ false);

  // ✅ Good
  pub enum Sides { Both, Single }
  pub enum Output { BlackAndWhite, Color }
  fn print_page(sides: Sides, color: Output) { /* ... */ }
  ```
- **Use enums with data fields** (algebraic data types) to model states that carry distinct payloads. Exhaustive `match` ensures every variant is handled.
- **All `match` arms must be explicit.** Avoid catch-all `_ =>` arms unless there is a strong justification; exhaustive matching catches future variant additions at compile time.

### 1.2 Express Behavior with Traits (Item 2)
- Use traits to define shared behavior across types.
- Derive standard traits (`Debug`, `Clone`, `PartialEq`, etc.) where appropriate. See Item 10.

### 1.3 Prefer `Option`/`Result` Transforms over Explicit `match` (Item 3)
- Use combinator methods (`.map()`, `.and_then()`, `.unwrap_or_else()`, `.ok_or()`) to chain transformations on `Option` and `Result`.
- **Use the `?` operator** to propagate errors concisely instead of nesting `match` expressions.
- Use `if let` when only the `Some`/`Ok` arm is relevant and you can ignore the other.
  ```rust
  // ❌ Verbose
  let value = match opt {
      Some(v) => v,
      None => return Err(MyError::Missing),
  };

  // ✅ Idiomatic
  let value = opt.ok_or(MyError::Missing)?;
  ```

### 1.4 Idiomatic Error Types (Item 4)
- **Implement `std::error::Error`** for custom error types. This requires `Display` and `Debug`.
- **Use `thiserror`** for library-style enum errors with nested variants and automatic `From` conversions.
- **Do NOT use `anyhow`**. This project prefers explicit, typed errors.
- When multiple error types are in play, unify them in an enum:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum AppError {
      #[error("database error: {0}")]
      Db(#[from] sqlx::Error),
      #[error("not found: {0}")]
      NotFound(String),
  }
  ```

### 1.5 Type Conversions (Item 5)
- Prefer explicit conversions via `From`/`Into` trait implementations over raw `as` casts.
- Implement `From<T>` for lossless conversions; the blanket `Into` impl comes for free.
- Use `TryFrom`/`TryInto` for conversions that can fail.
- **Avoid `as` casts** for numeric types when precision loss or sign change is possible; use `try_into()` instead.

### 1.6 Embrace the Newtype Pattern (Item 6)
- Wrap primitive types in single-field tuple structs to give them distinct type identity.
- Newtypes prevent accidental mixing of semantically different values (e.g., `UserId(i64)` vs `CommentId(i64)`).
- Use newtypes to bypass the orphan rule for implementing external traits on external types.
  ```rust
  pub struct UserId(pub i64);
  pub struct CommentId(pub i64);
  ```

### 1.7 Builders for Complex Types (Item 7)
- Use the builder pattern for structs with many fields, especially those with optional values.
- Use `Default` + struct update syntax (`..Default::default()`) for simpler cases.

### 1.8 References and Pointer Types (Item 8)
- Prefer `&T` (shared reference) and `&mut T` (unique reference) for borrowing data.
- Use `Box<T>` for heap allocation when ownership transfer or recursive types are needed.
- Use `Arc<T>` for shared ownership across threads. Combine with `Mutex<T>` or `RwLock<T>` when mutation is needed.
- **Avoid raw pointers** unless absolutely necessary (FFI, performance-critical unsafe code).

### 1.9 Iterator Transforms over Explicit Loops (Item 9)
- **Prefer iterator chains** (`.iter()`, `.map()`, `.filter()`, `.collect()`) over manual `for` loops with mutable accumulators.
- Iterator chains express intent more clearly and enable compiler optimizations.
- **Exception:** use explicit loops when the iterator chain becomes deeply nested, unreadable, or requires complex control flow (`break`, `continue` with labels).
  ```rust
  // ❌ Imperative
  let mut results = Vec::new();
  for item in items {
      if item.is_valid() {
          results.push(item.transform());
      }
  }

  // ✅ Functional
  let results: Vec<_> = items.iter()
      .filter(|item| item.is_valid())
      .map(|item| item.transform())
      .collect();
  ```

---

## 2. Traits

### 2.1 Standard Traits (Item 10)
- **Always derive** `Debug` for all public types.
- Derive `Clone`, `PartialEq`, `Eq`, `Hash` when semantically appropriate.
- Implement `Display` for types that should have human-readable representations.
- Implement `Default` when there is a natural "zero value."
- Implement `From`/`TryFrom` for type conversions (see Item 5).

### 2.2 RAII with `Drop` (Item 11)
- Implement `Drop` for types that own resources (connections, file handles, locks).
- Prefer RAII (Resource Acquisition Is Initialization) over manual cleanup.

### 2.3 Generics vs. Trait Objects (Item 12)
- **Prefer generics** (monomorphization) for performance-sensitive or small APIs.
- **Use trait objects** (`dyn Trait`) when you need heterogeneous collections, need to reduce binary size, or the concrete type isn't known at compile time.
- Understand the trade-off: generics = faster but more code generated; trait objects = smaller binary with vtable overhead.

### 2.4 Default Implementations (Item 13)
- Provide default method implementations in traits to minimize the number of methods a type must implement.
- Override defaults only when the custom behavior differs from the provided default.

---

## 3. Concepts

### 3.1 Lifetimes (Item 14)
- Let lifetime elision handle most cases. Add explicit lifetime annotations only when the compiler requires them.
- Prefer owned types (`String`, `Vec<T>`) over references in struct fields unless there's a clear performance benefit.

### 3.2 Borrow Checker (Item 15)
- Understand the rules: one mutable reference OR multiple shared references, never both.
- Design APIs that minimize borrow conflicts. Consider cloning when fighting the borrow checker for marginal performance gains.

### 3.3 Avoid `unsafe` Code (Item 16)
- **Do not use `unsafe` unless absolutely required** (e.g., FFI boundaries).
- If `unsafe` is unavoidable, encapsulate it in a safe abstraction and document the invariants that must hold in a `# Safety` section.
- Mark `unsafe` functions as such and require callers to acknowledge the contract.

### 3.4 Shared-State Parallelism (Item 17)
- Be wary of shared mutable state. Prefer message passing (`mpsc`, `tokio::sync::mpsc`) over shared memory with locks.
- When locks are necessary, keep the critical section as short as possible.
- Beware of deadlocks; always acquire locks in a consistent order.

### 3.5 Don't Panic (Item 18)
- **Avoid `unwrap()` and `expect()` in non-test production code.** Use `?` to propagate errors.
- **Avoid `panic!()` in library code.** Return `Result<T, E>` instead.
- `unwrap()` / `expect()` are acceptable in:
  - Test code (`#[cfg(test)]`).
  - Cases where the invariant is provably guaranteed (document *why* with a comment).
  - One-shot scripts or `main()` where there's no caller to propagate to.
- If providing both fallible and infallible APIs, document the panic conditions in a `# Panics` doc section.

### 3.6 Avoid Reflection (Item 19)
- Don't use `Any` / downcasting for runtime type checks. Encode variants in the type system with enums or generics.

### 3.7 Don't Over-Optimize (Item 20)
- Write clear, correct code first. Optimize only after profiling identifies a bottleneck.
- Prefer algorithmic improvements over micro-optimizations.
- Trust the compiler and iterator pipeline; it often optimizes better than hand-written loops.

---

## 4. Dependencies

### 4.1 Semantic Versioning (Item 21)
- Pin dependencies to compatible versions in `Cargo.toml` (e.g., `"1.2"` not `"*"`).
- Understand what semver permits: minor versions may add new public API items.

### 4.2 Minimize Visibility (Item 22)
- **Default to private.** Only make items `pub` when they're part of the module's API.
- Use `pub(crate)` for items shared internally within the crate but not exposed externally.
- Use `pub(super)` when an item should be visible only to the parent module.
- Don't export internal helpers by default.
  ```rust
  // ❌ Over-exposed
  pub fn internal_helper() { /* ... */ }

  // ✅ Scoped visibility
  pub(crate) fn internal_helper() { /* ... */ }
  ```

### 4.3 Avoid Wildcard Imports (Item 23)
- **Do not use `use some_crate::*`** for external crates. It makes name resolution fragile and hides where types come from.
- Wildcard imports are acceptable for:
  - The `prelude` pattern within your own crate.
  - Test modules importing from the parent (`use super::*`).
  - Enum variants within a `match` when the enum is local.

### 4.4 Re-Export Dependencies in Your API (Item 24)
- If a public function or struct exposes a type from a dependency, re-export that dependency's type so users don't need to add it to their own `Cargo.toml`.

### 4.5 Manage Your Dependency Graph (Item 25)
- Favor well-maintained, widely-used crates.
- Add dependencies to `Cargo.toml` with **minimal features** enabled.
- For workspace members, declare dependencies in the root `Cargo.toml` and reference them with `{ workspace = true }`.
- Pass dependencies explicitly as parameters (e.g., `&SqlitePool`, `&Client`). **Avoid singletons and global state.**

### 4.6 Feature Creep (Item 26)
- Don't enable features you don't need. Each feature increases compilation time and attack surface.
- If your crate offers features, keep them orthogonal and well-documented.

---

## 5. Tooling

### 5.1 Document Public Interfaces (Item 27)
- Use `///` doc comments for all public items (structs, enums, traits, functions, methods).
- Use `//!` for module-level and crate-level documentation.
- Include a `# Examples` section with runnable code snippets where helpful.
- Document `# Panics` conditions and `# Safety` requirements for `unsafe` functions.
- Use `[`SomeType`]` syntax for cross-references in doc comments.
- **Comments must add information**, not just restate what the code does.
  ```rust
  /// Fetches comments for the given URL, ordered by creation date.
  ///
  /// # Errors
  /// Returns `AppError::Db` if the database query fails.
  /// Returns `AppError::NotFound` if the URL doesn't exist.
  pub async fn get_comments(url_id: i64, pool: &PgPool) -> Result<Vec<Comment>, AppError> {
      // ...
  }
  ```

### 5.2 Use Macros Judiciously (Item 28)
- Prefer functions and generics over macros. Use macros only when they're the **only way** to eliminate boilerplate or keep disparate code in sync.
- Macro invocations should look like normal Rust code or be sufficiently distinct that they can't be confused.
- Avoid nonlocal control flow in macros (e.g., hidden `return` or `?`).
- Before writing a derive macro, check if an existing crate provides it.

### 5.3 Listen to Clippy (Item 29)
- **Run `cargo clippy --all-targets --all-features -D warnings`** before every commit and after finishing writing code for a change.
- Fix warnings rather than suppressing them. If a suppression is necessary, scope it to the smallest block and add a comment explaining why. **Present any remaining warnings to the user when done with the implementation.**
  ```rust
  // This cast is safe because the value is always within u32 range.
  #[allow(clippy::cast_possible_truncation)]
  let count = total as u32;
  ```
- Make the codebase **Clippy-warning free** so new warnings are immediately visible.

### 5.4 Write More Than Unit Tests (Item 30)
- **Unit tests** (`#[cfg(test)]` in the same file): test internal logic, including private functions.
- **Integration tests** (`tests/` directory): exercise the public API end-to-end.
- **Doc tests** (code in `///` comments): verify examples compile and run correctly.
- When fixing a bug, **write a test that reproduces the bug first**, then fix it.

### 5.5 Tooling Ecosystem (Item 31)
- Use `rustfmt` for consistent formatting. Run `cargo fmt --all` before committing.
- Use `cargo clippy` as described above.
- Use `cargo doc --open` to review generated documentation.
- Use `cargo test` to run all tests (unit, integration, doc tests).

### 5.6 Continuous Integration (Item 32)
- CI must run: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, and `cargo doc`.
- Fail the build on any warning or test failure.

---

## 6. Naming & Style Conventions

These conventions are specific to this project and complement the Effective Rust guidelines:

| Element | Convention | Example |
|---------|-----------|---------|
| Functions, variables, modules | `snake_case` | `get_comments`, `url_id` |
| Types, traits, enums | `CamelCase` | `AppState`, `LinksAppState` |
| Constants, statics | `SCREAMING_SNAKE_CASE` | `MAX_RETRIES`, `DEFAULT_TIMEOUT` |
| Crate names | `lowercase` (with hyphens) | `web-server`, `api-gen` |
| Lifetimes | Short, lowercase | `'a`, `'ctx` |
| Type parameters | Single uppercase or descriptive | `T`, `E`, `Repo` |

### Additional Project Rules
- **Don't rearrange existing function parameters** onto new lines. Only restructure when adding new parameters.
- **Don't reformat code** unless specifically asked.
- **Formatting tool:** `cargo fmt --all` — keep `rustfmt` defaults.

---

## 7. Error Handling Summary

```
                    ┌──────────────────────────────┐
                    │    Can the operation fail?    │
                    └──────────────┬───────────────┘
                           YES    │
                    ┌─────────────▼──────────────┐
                    │  Return Result<T, E>       │
                    │  Propagate with `?`        │
                    └─────────────┬──────────────┘
                                  │
              ┌───────────────────┴───────────────────┐
              │ Multiple error types?                 │
              └───────────┬───────────────────────────┘
                    YES   │                      NO
         ┌────────────────▼──────────┐   ┌───────────▼────────┐
         │  Enum with #[from] via    │   │  Return the single │
         │  thiserror                │   │  error type         │
         └───────────────────────────┘   └────────────────────┘
```

---

## 8. Async & Concurrency

- Use `tokio` as the async runtime.
- Prefer `async fn` and `.await` over spawning manual threads.
- **Bound concurrency** with `Semaphore` or `FuturesUnordered`; never spawn unbounded tasks.
- **Don't block** on async threads. Wrap blocking I/O or CPU-heavy work in `tokio::task::spawn_blocking`.
- Use `tokio::sync::mpsc` for message passing between tasks.

---

## 9. Logging

- Use the `log` crate macros: `trace!`, `debug!`, `info!`, `warn!`, `error!`.
- Keep log messages **structured and actionable**:
  ```rust
  // ❌ Bad
  info!("done");

  // ✅ Good
  info!(url_id = %id, count = comments.len(), "fetched comments");
  ```
- **Never log secrets**, tokens, passwords, or PII.

---

## 10. Quick Reference Checklist

Before submitting code, verify:

- [ ] `cargo fmt --all` — no formatting diffs
- [ ] `cargo clippy --all-targets --all-features -D warnings` — no warnings
- [ ] `cargo test` — all tests pass
- [ ] No `unwrap()` / `expect()` in non-test code
- [ ] No `anyhow` usage — only `thiserror` if needed
- [ ] Public items have `///` doc comments
- [ ] Visibility is minimized (`pub(crate)` over `pub` when possible)
- [ ] No wildcard imports from external crates
- [ ] Dependencies added with minimal features
- [ ] New errors are encoded in typed enums, not stringly typed

---

*Source: [Effective Rust](https://effective-rust.com/title-page.html) by David Drysdale — 35 Specific Ways to Improve Your Rust Code*
