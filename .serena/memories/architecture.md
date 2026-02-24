# Architecture & Design Patterns

## Repository Trait Pattern
- All DB access through dedicated traits (e.g., `PgCommentsRepository`).
- `CombinedRepository` is the blanket supertrait.
- Testing uses mocks of these traits.

## Task Deduplication Queue
- `TaskDedupQueue<T>` manages unique tasks via `Hash + Eq`.
- Sequential processing in a dedicated worker loop.

## Axum State Extraction
- `AppState` is the top-level state.
- Handler-specific substates implement `FromRef<AppState>`.
