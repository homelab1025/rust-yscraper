# Implementation Plan: Extract and Display Subcomment Count

## Phase 1: Database and Backend Core [checkpoint: 1171cc938b7c570643311fe93000f140f8faf5ed]

1.  - [x] Task: Create Liquibase migration to add `subcomment_count` to `comments` table.
    - [x] Create `db/changelog/005_add_subcomment_count.sql`.
    - [x] Update `db/changelog/db.changelog-master.yaml`.

    **Summary:** Created a new Liquibase migration to add a `subcomment_count` column to the `comments` table. This column is an integer, defaults to 0, and is non-nullable.
    - `db/changelog/005_add_subcomment_count.sql`
    - `db/changelog/db.changelog-master.yaml`
2.  - [x] Task: Update Backend Entities and Repository.
    - [x] Update `CommentRecord` struct in `web_server/src/lib.rs` to include `subcomment_count`.
    - [x] Update `DbCommentRow` struct in `web_server/src/db/comments_repository.rs` to include `subcomment_count`.
    - [x] Update `PgCommentsRepository` in `web_server/src/db/postgresql.rs` to handle the new column in SQL queries (select/insert/update).

    **Summary:** Updated the `CommentRecord` and `DbCommentRow` structs to include the `subcomment_count` field. Modified `PgCommentsRepository` to fetch and store the `subcomment_count` in the database.
    - `web_server/src/lib.rs`
    - `web_server/src/db/comments_repository.rs`
    - `web_server/src/db/postgresql.rs`
3.  - [x] Task: Update Scraping Logic.
    - [x] Modify `web_server/src/scrape.rs` to extract the `n` attribute from elements with classes `.togg.clicky`.
    - [x] Ensure it defaults to `0` if not found.

    **Summary:** Updated the `parse_root_comments` function in `web_server/src/scrape.rs` to extract the `n` attribute from the `.togg.clicky` element, which represents the number of subcomments.
    - `web_server/src/scrape.rs`
4.  - [x] Task: Write Tests for Scraping Logic.
    - [x] Add unit tests in `web_server/src/scrape.rs` using HTML fixtures (some existing, maybe create a new one) to verify subcomment extraction.

    **Summary:** Added a unit test `get_comments_extracts_subcomment_count` to `web_server/src/scrape.rs` and a new HTML fixture `web_server/tests/fixtures/hn_subcomments.html`. Verified that the subcomment count is correctly extracted and defaults to 0 when missing.
    - `web_server/src/scrape.rs`
    - `web_server/tests/fixtures/hn_subcomments.html`
5.  - [x] Task: Conductor - User Manual Verification 'Phase 1: Database and Backend Core' (Protocol in workflow.md)

## Phase 2: API and Client Regeneration [checkpoint: 01fc7520385b0feaa17de510bb1e064bcacc12f9]

1.  - [x] Task: Update API Handlers and OpenAPI Documentation.
    - [x] Ensure `subcomment_count` is correctly returned in the JSON response in `web_server/src/api/comments.rs`.
    - [x] Verify `utoipa` annotations on the `Comment` struct.

    **Summary:** Verified that `CommentDto` in `web_server/src/api/comments.rs` includes `subcomment_count` and is correctly annotated with `utoipa::ToSchema`. The `list_comments` handler now returns this field.
    - `web_server/src/api/comments.rs`
2.  - [x] Task: Regenerate API Client.
    - [x] Run `cargo run -p api_gen -- openapi.yaml` to generate the updated OpenAPI spec.
    - [x] Run `npm run generate-api` in the `webapp` directory to update the TypeScript client.

    **Summary:** Regenerated the `openapi.yaml` file, copied it to `webapp/openapi.yaml`, and ran `npm run generate-api` in the `webapp` directory. The new client now correctly includes the `subcomment_count` field in `CommentDto`.
    - `openapi.yaml`
    - `webapp/openapi.yaml`
    - `webapp/src/api-client/` (multiple files)
3.  - [x] Task: Conductor - User Manual Verification 'Phase 2: API and Client Regeneration' (Protocol in workflow.md)

## Phase 3: Frontend Implementation

1.  - [ ] Task: Update Frontend to display Subcomments.
    - [ ] Modify `webapp/src/pages/CommentsPage.tsx` to add a new "Subcomments" column after the "Author" column.
    - [ ] Update `webapp/src/components/CommentRow.tsx` (if it exists) to display the new data.
2.  - [ ] Task: Verify Frontend with Local Dev Stack.
    - [ ] Run the full stack and verify that scraped comments show the subcomment count in the UI.
3.  - [ ] Task: Conductor - User Manual Verification 'Phase 3: Frontend Implementation' (Protocol in workflow.md)
