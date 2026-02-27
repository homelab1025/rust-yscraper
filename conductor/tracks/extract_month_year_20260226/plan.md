# Implementation Plan - Extract Month and Year from Thread Title

## Phase 1: Database & Backend Infrastructure [checkpoint: 00ef5fe970665ca4b66a75af4d077154c1dce2b6]
- [x] Task: Create Liquibase migration to add `thread_month` and `thread_year` to `urls` table
    - [x] Create `db/changelog/006_add_thread_month_year.sql`.
    - [x] Update `db/changelog/db.changelog-master.yaml`.
    - **Summary**: Added `thread_month` (INTEGER) and `thread_year` (INTEGER) columns to the `urls` table via a new Liquibase migration. Created `db/changelog/006_add_thread_month_year.sql` and updated `db/changelog/db.changelog-master.yaml`.
- [x] Task: Update Database Models and Repository
    - [x] Update `DbUrlRow` and `ScheduledUrl` in `web_server/src/db/links_repository.rs`.
    - [x] Update `LinksRepository` trait with `thread_month` and `thread_year` support.
    - [x] Update `PgCommentsRepository` in `web_server/src/db/postgresql.rs` for CRUD operations.
    - **Summary**: Updated `DbUrlRow` and `ScheduledUrl` structs to include `thread_month` and `thread_year`. Added and implemented `update_thread_metadata` in `LinksRepository` and `PgCommentsRepository`. Updated relevant SQL queries to include the new columns.
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: Scraper Enhancement (TDD) [checkpoint: 7e9a60c9f4f8fddff86b8ba8eda1bf81869ab764]
- [x] Task: Implement Month/Year extraction in `web_server/src/scrape.rs`
    - [x] Write unit tests for extraction with various title formats.
    - [x] Implement regex extraction logic.
    - [x] Update `get_comments` (or related flow) to return extracted month/year.
    - **Summary**: Implemented `extract_month_year` using regex. Updated it to return a `Result` and log errors at the ERROR level if extraction fails. Updated `get_comments` to handle the metadata and return `ScrapeResult`. Added comprehensive unit tests.
- [x] Task: Integrate extraction into Scrape Task
    - [x] Update `ScrapeTask` in `web_server/src/scrape_task.rs` to store extracted metadata.
    - **Summary**: Refactored `LinksRepository` by merging `update_last_scraped` into `update_thread_metadata`, which now updates the `last_scraped` timestamp along with the thread month and year in a single query. Maintained `update_comment_count` as a separate method that only updates `picked_comment_count`. Updated `ScrapeTask::execute` to use these methods after a successful scrape.
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: API & Frontend Integration [checkpoint: f478189ca883b0ed4826c5b85980561c2b543f63]
- [x] Task: Update API Endpoints
    - [x] Update `LinkDto` in `web_server/src/api/links.rs`.
    - [x] Ensure `list_links` returns the new fields.
    - **Summary**: Updated `LinkDto` to include `thread_month` and `thread_year`. Modified `list_links` to map these fields from the database. Updated API tests.
- [x] Task: Update Frontend Links Table
    - [x] Update `webapp/src/pages/LinkManagementPage.tsx` to handle `thread_month` and `thread_year`.
    - [x] Implement month integer to name mapping.
    - [x] Update link display logic in the table.
    - **Summary**: Updated `LinkManagementPage.tsx` to display "Month Year" as link text when metadata is available. Implemented `formatThreadMetadata` helper. Used type casting to bypass temporary generated type limitations.
- [x] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
