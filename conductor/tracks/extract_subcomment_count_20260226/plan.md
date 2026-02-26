# Implementation Plan: Extract and Display Subcomment Count

## Phase 1: Database and Backend Core

1.  - [x] Task: Create Liquibase migration to add `subcomment_count` to `comments` table.
    - [x] Create `db/changelog/005_add_subcomment_count.sql`.
    - [x] Update `db/changelog/db.changelog-master.yaml`.

    **Summary:** Created a new Liquibase migration to add a `subcomment_count` column to the `comments` table. This column is an integer, defaults to 0, and is non-nullable.
    - `db/changelog/005_add_subcomment_count.sql`
    - `db/changelog/db.changelog-master.yaml`
2.  - [ ] Task: Update Backend Entities and Repository.
    - [ ] Update `Comment` struct in `web_server/src/db/comments_repository.rs` to include `subcomment_count`.
    - [ ] Update `CommentsRepository` trait methods if necessary.
    - [ ] Update `PgCommentsRepository` in `web_server/src/db/postgresql.rs` to handle the new column in SQL queries (select/insert/update).
3.  - [ ] Task: Update Scraping Logic.
    - [ ] Modify `web_server/src/scrape.rs` to extract the `n` attribute from elements with classes `.togg.clicky`.
    - [ ] Ensure it defaults to `0` if not found.
4.  - [ ] Task: Write Tests for Scraping Logic.
    - [ ] Add unit tests in `web_server/src/scrape.rs` using HTML fixtures (some existing, maybe create a new one) to verify subcomment extraction.
5.  - [ ] Task: Conductor - User Manual Verification 'Phase 1: Database and Backend Core' (Protocol in workflow.md)

## Phase 2: API and Client Regeneration

1.  - [ ] Task: Update API Handlers and OpenAPI Documentation.
    - [ ] Ensure `subcomment_count` is correctly returned in the JSON response in `web_server/src/api/comments.rs`.
    - [ ] Verify `utoipa` annotations on the `Comment` struct.
2.  - [ ] Task: Regenerate API Client.
    - [ ] Run `cargo run -p api_gen -- openapi.yaml` to generate the updated OpenAPI spec.
    - [ ] Run `npm run generate-api` in the `webapp` directory to update the TypeScript client.
3.  - [ ] Task: Conductor - User Manual Verification 'Phase 2: API and Client Regeneration' (Protocol in workflow.md)

## Phase 3: Frontend Implementation

1.  - [ ] Task: Update Frontend to display Subcomments.
    - [ ] Modify `webapp/src/pages/CommentsPage.tsx` to add a new "Subcomments" column after the "Author" column.
    - [ ] Update `webapp/src/components/CommentRow.tsx` (if it exists) to display the new data.
2.  - [ ] Task: Verify Frontend with Local Dev Stack.
    - [ ] Run the full stack and verify that scraped comments show the subcomment count in the UI.
3.  - [ ] Task: Conductor - User Manual Verification 'Phase 3: Frontend Implementation' (Protocol in workflow.md)
