# Specification: Comment Sorting

## Overview
Enable users to sort comments on the Comments page by their creation date and the number of subcomments they have received. This requires changes to the backend API to support sorting parameters and updates to the frontend UI to provide interactive headers.

## Functional Requirements

### Backend (Rust/Axum)
- **API Extension**: Update the comments retrieval endpoint (likely `/api/links/{id}/comments`) to accept optional query parameters:
  - `sort_by`: `date` (default) or `subcomment_count`.
  - `order`: `asc` or `desc` (default).
- **Database Query**: Modify the PostgreSQL query in `comments_repository.rs` to incorporate `ORDER BY` clauses based on the provided parameters.

### Frontend (React/Material UI)
- **Interactive Headers**: Make the "Date" and "Subcomments" table headers clickable.
- **Sort Logic**:
  - Clicking the current sort header toggles the order (`desc` <-> `asc`).
  - Clicking a different header switches the sort column and defaults the order to `desc`.
- **Visual Feedback**:
  - Display sort direction icons (e.g., `ArrowUpward`, `ArrowDownward`).
  - Highlight the active sort column header to distinguish it from inactive ones.
- **Default State**: On initial load, the comments should be sorted by **Date DESC** (newest first).

## Non-Functional Requirements
- **Performance**: Sorting should be efficient at the database level.
- **Consistency**: The UI state must reflect the actual data returned by the API.

## Acceptance Criteria
- [ ] User can click the "Date" header to toggle between newest-first and oldest-first.
- [ ] User can click the "Subcomments" header to toggle between most-engaged and least-engaged.
- [ ] Sorting correctly switches back to "Date DESC" if the page is refreshed or re-entered.
- [ ] The API correctly handles all combinations of `sort_by` and `order`.
- [ ] The UI displays clear visual indicators (icons and highlighting) for the current sort.

## Out of Scope
- Sorting by other fields (e.g., author, content).
- Client-side sorting (all sorting must be handled by the backend).
