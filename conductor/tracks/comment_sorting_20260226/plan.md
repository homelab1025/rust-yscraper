# Implementation Plan: Comment Sorting

This plan covers the end-to-end implementation of sorting functionality for comments, allowing users to sort by date and subcomment count in both ascending and descending orders.

## Phase 1: Backend - Repository Layer (TDD) [checkpoint: 7eba456a596c1918d9239d29db4d8e15ad7a45d8]
Focus: Implementing the sorting logic at the database query level with full test coverage.

- [x] Task: Create `web_server/tests/sorting_test.rs` to define expected sorting behavior.
- [x] Task: Update `CommentsRepository` trait in `web_server/src/db/comments_repository.rs` to include sorting parameters in `page_comments`.
- [x] Task: Implement sorting logic in `PgCommentsRepository::page_comments` in `web_server/src/db/postgresql.rs`.
- [x] Task: Verify that all repository tests pass, including the new sorting tests.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Backend - Repository Layer' (Protocol in workflow.md)

## Phase 2: Backend - API Layer
Focus: Exposing the sorting functionality through the REST API.

- [x] Task: Update `CommentsFilter` struct in `web_server/src/api/comments.rs` to include `sort_by` and `order` parameters.
- [x] Task: Update `list_comments` handler in `web_server/src/api/comments.rs` to pass sorting parameters to the repository.
- [x] Task: Update OpenAPI documentation in `web_server/src/api/comments.rs`.
- [x] Task: Add integration tests for sorting in `web_server/src/api/comments.rs` (MockedRepo).
- [x] Task: Conductor - User Manual Verification 'Phase 2: Backend - API Layer' (Protocol in workflow.md)
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Backend - API Layer' (Protocol in workflow.md)

## Phase 3: Frontend - Data Fetching & State
Focus: Updating the frontend to handle sort state and fetch sorted data.

- [ ] Task: Update API fetching logic in `webapp/src/pages/CommentsPage.tsx` to include `sort_by` and `order` in the request.
- [ ] Task: Define state for `sortColumn` and `sortDirection` in `CommentsPage.tsx`.
- [ ] Task: Implement the sorting toggle logic (DESC -> ASC -> DESC, default to DESC on new column).
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Frontend - Data Fetching & State' (Protocol in workflow.md)

## Phase 4: Frontend - UI Implementation
Focus: Making the UI interactive and providing visual feedback for sorting.

- [ ] Task: Update table headers in `CommentsPage.tsx` to be clickable.
- [ ] Task: Add `TableSortLabel` (Material UI) or equivalent to "Date" and "Subcomments" headers.
- [ ] Task: Ensure active column is highlighted and shows the correct direction icon.
- [ ] Task: Final verification of the end-to-end flow: clicking headers refreshes the list with correct sorting.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Frontend - UI Implementation' (Protocol in workflow.md)
