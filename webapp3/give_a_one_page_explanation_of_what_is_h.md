### What `App.tsx` does (and the React + TypeScript concepts it uses)

`App.tsx` is a single React function component that renders a paginated table of comments from the backend API and adds keyboard-driven navigation and local row marking. It’s written in TypeScript, so state, props, and refs are strongly typed.

#### Data types and API shape
- Types live in `src/types.ts`:
  - `CommentDto` is the shape of each row (`id`, `text`, `user`, `url_id`, `date`).
  - `CommentsPage` wraps a page response: `{ total, items }`.
- `useApiBase()` reads `import.meta.env.VITE_API` (Vite’s env) and returns a base URL string (or empty for same-origin/proxy).

#### State management with hooks
- `useState` holds UI and data state:
  - `items: CommentDto[]`, `total: number` — the current page of data and total count.
  - `page: number`, `pageSize: number` — the current pagination cursor (page size is fixed to 10 here).
  - `loading: boolean` — request-in-flight flag.
  - `selectedIndex: number | null` — which table row is highlighted/active.
  - `marks: Record<number, 'blue' | 'red'>` — local-only color marks keyed by `comment.id` using a TypeScript index signature and string-literal union type.

#### Refs for DOM access (scrolling into view)
- `scrollerRef = useRef<HTMLDivElement | null>(null)` points to the scrollable container div.
- `rowRefs = useRef<Array<HTMLTableRowElement | null>>([])` stores per-row DOM nodes in an array parallel to `items`.
- `setRowRef(idx)` is a memoized callback that assigns each row element into `rowRefs.current[idx]`.

#### Derived values with `useMemo`
- `totalPages = Math.ceil(total / pageSize)` — recomputed only when inputs change.
- `pagesToShow` — a small sliding window of page numbers around the current page to keep pagination compact.

#### Fetching data with `useCallback` + `useEffect`
- `load` is an `async` function memoized with `useCallback` that:
  - Builds `offset` from `page`/`pageSize` and fetches `${apiBase}/comments?offset=&count=`.
  - Parses JSON as `CommentsPage`, resets `rowRefs`, updates `items` and `total`.
  - Resets the scroller’s `scrollTop` to 0 for a new page; handles errors with a safe fallback.
- A `useEffect([load])` runs `load()` on mount and whenever its dependencies (API base/page) change.

#### Keeping selection valid and visible
- After `items` change, another `useEffect` clamps `selectedIndex` into bounds (or clears it if no rows) and then triggers `ensureRowInView(selectedIndex)` via `requestAnimationFrame` so it runs after the DOM updates.
- `ensureRowInView(idx, buffer=2)` scrolls the container so the selected row plus a small buffer stay visible. It computes desired scroll based on `offsetTop` and `clientHeight` of top/bottom buffer rows and calls `container.scrollTo({ behavior: 'smooth' })` when needed.

#### Keyboard interaction (global)
- A `useEffect` attaches a `keydown` listener with cleanup on unmount.
- Guard clauses avoid handling keys while typing in inputs and while `loading`.
- Supported keys:
  - `j` — move selection down (bounds checked).
  - `k` — move selection up.
  - `a` — mark the selected row blue: `setMarks(m => ({ ...m, [id]: 'blue' }))`.
  - `d` — mark red similarly. These marks are local UI state only (no server calls).
- After `j/k`, it schedules `ensureRowInView` so the highlight remains visible.

#### Rendering
- Header and a main content section with:
  - Conditional loading state vs. the table.
  - A table whose rows come from `items.map(...)`.
  - Click on a row sets `selectedIndex` and scrolls it into view.
  - Row CSS classes computed by `rowClass(c, idx)` to apply `selected`, `mark-blue`, or `mark-red`.
  - Pagination controls: Prev/Next buttons and a few page-number buttons; disabled states are derived from `page` and `totalPages`.
- Accessibility: `aria-label` on the table, `aria-selected` on rows, and `role="navigation"` on pagination.

#### TypeScript highlights
- Strongly typed state: e.g., `useState<CommentDto[]>([])`, `useState<Record<number, 'blue' | 'red'>>({})`.
- Typed refs: `useRef<HTMLDivElement | null>(null)` / `useRef<HTMLTableRowElement | null>(null)` ensure DOM usage is safe.
- Narrowing and guards for `null`/`undefined` (e.g., `idx == null` early returns, `selectedIndex != null` checks).
- Environment variable cast: `(import.meta.env.VITE_API as string | undefined)` — Vite’s `import.meta.env` is typed, but this narrows to an optional string.

#### Performance and ergonomics
- `useMemo` and `useCallback` prevent unnecessary recalculation/re-creation across renders (useful for stable refs and efficient children rendering).
- Deferring scroll logic using `requestAnimationFrame`/`setTimeout` avoids reading stale layout before the DOM paints.
- Row refs array is reset on each load to match the new `items` list and avoid mismatches.

In short, `App.tsx` is a typed, self-contained React view layer that fetches paginated comments, manages selection/marking in local state, keeps the selected row visible via scroll math, and exposes minimal, accessible controls for paging and keyboard navigation. It uses idiomatic hooks (`useState`, `useEffect`, `useMemo`, `useCallback`, `useRef`) and TypeScript’s type system to make DOM and state interactions safe and predictable.