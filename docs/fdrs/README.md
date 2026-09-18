# Functional Decision Records (FDRs)

This folder records significant **functional/product** decisions — changes to what the product does or how a
user-facing workflow behaves — the same way [`docs/adrs/`](../adrs/) records architectural decisions.

## When to write one

Add an FDR when a PR changes user-facing behavior or product scope in a way that isn't obvious from the diff, and
a future contributor would benefit from knowing *why*. Routine bug fixes and behavior-preserving refactors don't
need one.

## Format

```markdown
# FDR NNNN: <title>

## Status
Proposed | Accepted | Superseded

## Context
What functional problem or requirement prompted this decision?

## Decision
What was decided, from the user/product point of view?

## Consequences
Trade-offs, follow-up work, or edge cases to watch out for.
```

Number sequentially (`0001-<slug>.md`, `0002-<slug>.md`, ...), matching the convention in `docs/adrs/`.
