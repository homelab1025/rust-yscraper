# Contributing to rust-yscraper

This is the entry point for contributing to the project. For per-language style rules see
[`contributing/code_styleguides/`](contributing/code_styleguides/), and for product/stack context see
[`contributing/product.md`](contributing/product.md) and [`contributing/tech-stack.md`](contributing/tech-stack.md).
This file covers process: how to work, test, and get a PR merged.

## Getting started

- `cargo run -p web_server` runs the API server (needs `config.toml` in the CWD and a running Postgres).
- `cd webapp && npm install && npm run dev` runs the frontend dev server.
- `./dev.sh start` brings up the full local stack (Postgres + Liquibase migrations + `web_server` + webapp) for
  the current worktree in one command; `./dev.sh status` / `./dev.sh stop` manage it. See the root `README.md`
  and `CLAUDE.md` for the full command reference.

## Working in worktrees

Prefer a dedicated `git worktree` per feature/fix branch instead of switching branches in place, especially when
you need to keep working on something else in parallel. `dev.sh` derives its ports and Postgres data volume from
the worktree's absolute path, so multiple worktrees can each run their own local stack at the same time without
colliding — no manual port bookkeeping needed.

## dev.sh

[`docs/dev-sh.md`](docs/dev-sh.md) documents the flow of the `dev.sh` script. Whenever you change `dev.sh`
(new commands, changed ports/offsets, new state files, changed startup/stop behavior), update that document in
the same PR so the two stay in sync.

## Testing

- **Unit tests** live alongside the code (`#[cfg(test)]` modules) and use mocked repository traits — no database
  needed. Run with `cargo test -p web_server --lib`.
- **Integration tests** live under `web_server/tests/` (one file = one test binary, e.g. `postgresql_test.rs`,
  `comments_api_test.rs`, `links_api_test.rs`, `sorting_test.rs`, `ping_integration.rs`). They spin up a real
  Postgres via `testcontainers`, so they need a Docker daemon reachable from the process. **Plain `cargo test`
  runs both unit and integration binaries together** — there's no separate flag that skips `tests/`.
- If a change touches query/filter logic, add or extend an integration test for it, not just a mocked unit test —
  mocks won't catch a wrong SQL clause. See `web_server/tests/comments_api_test.rs` for the pattern (seed rows via
  the pool, hit the router with a real request, assert on the response body).
- Docker discovery:
  - On native Linux (including our self-hosted CI runners), `testcontainers` auto-detects `/var/run/docker.sock`
    — no `DOCKER_HOST` needed, as long as the runner/user can access the socket (e.g. is in the `docker` group).
  - On macOS with Colima, set `DOCKER_HOST=unix://${HOME}/.colima/default/docker.sock` first.
  - If you genuinely can't reach Docker locally, say so plainly in the PR's test plan (see below) instead of
    silently skipping the integration tests — but don't rely on that as a substitute for running them before
    merge, since CI runs them on every push/PR via the `build` job in `.github/workflows/yscraper.yml`.

## Database changes

Schema changes go through Liquibase (`db/changelog/`): add a new changeset entry to `changelog-master.yaml` plus
its own SQL file. Don't edit previously-applied changeset SQL files in place.

## Generated code

The TypeScript API client under `webapp/src/lib/server/` is generated from the OpenAPI spec
(`cargo run -p api_gen -- openapi.yaml` then `openapi-generator generate -g typescript-axios ...` — see
`CLAUDE.md` for the exact commands) and must be regenerated, not hand-edited, whenever the API changes.

## Commit messages

Follow the existing convention: `type(scope): short description` (e.g. `fix(webapp): ...`, `feat(dev): ...`,
`ci(yscraper): ...`, `chore(hooks): ...`). Keep the body focused on *why*, not a restatement of the diff.

## Decision records

For every PR, check whether it introduces a significant change worth recording, and if so write it down rather
than leaving the reasoning only in the PR description:

- **Architectural changes** (new component, a changed data flow, one structural pattern swapped for another, a
  cross-cutting decision like the `comment_count` denormalization in `docs/adrs/0001-...md`) → add an ADR to
  [`docs/adrs/`](docs/adrs/), following the format of the existing entries (`NNNN-title.md`: Status, Context,
  Decision, Consequences).
- **Functional/product changes** (new user-facing behavior, a changed workflow, a scope decision about what the
  product does rather than how it's built) → add an FDR (Functional Decision Record) to
  [`docs/fdrs/`](docs/fdrs/), using the same lightweight format — see that folder's `README.md` for the template.

Not every PR needs one — routine bug fixes, small additions, and refactors that don't change behavior don't. Use
judgment: if a future contributor would benefit from knowing *why* a non-obvious structural or product decision
was made, write it down.

## Pull requests

- Keep the PR's **test plan checklist honest**: if something wasn't run locally (e.g. no Docker socket in your
  environment), say so explicitly rather than leaving an unchecked box with no explanation — and reconcile it
  once CI has actually run those tests, rather than leaving a stale caveat that no longer matches reality.
- CI (`.github/workflows/yscraper.yml`) runs `cargo test --locked` — unit *and* integration — plus a SonarCloud
  scan on every PR into `main`. The `build-web-server`/`build-webapp` image-publish jobs only run on pushes to
  `main`, not on PRs, so seeing them skipped on a PR is expected, not a failure.
- The PR must pass the **SonarCloud quality gate**. Fix flagged issues before merging where they're valid. If you
  disagree with one, don't just dismiss it silently — leave a PR comment explaining the reasoning for not fixing
  it (see PR #22 for an example of documenting an intentional "won't fix").
