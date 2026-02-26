# Implementation Plan - Extract Month and Year from Thread Title

## Phase 1: Database & Backend Infrastructure
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

## Phase 2: Scraper Enhancement (TDD)
- [ ] Task: Implement Month/Year extraction in `web_server/src/scrape.rs`
    - [ ] Write unit tests for extraction with various title formats.
    - [ ] Implement regex extraction logic.
    - [ ] Update `get_comments` (or related flow) to return extracted month/year.
- [ ] Task: Integrate extraction into Scrape Task
    - [ ] Update `ScrapeTask` in `web_server/src/scrape_task.rs` to store extracted metadata.
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: API & Frontend Integration
- [ ] Task: Update API Endpoints
    - [ ] Update `LinkDto` in `web_server/src/api/links.rs`.
    - [ ] Ensure `list_links` returns the new fields.
- [ ] Task: Update Frontend Links Table
    - [ ] Update `webapp/src/pages/LinkManagementPage.tsx` to handle `thread_month` and `thread_year`.
    - [ ] Implement month integer to name mapping.
    - [ ] Update link display logic in the table.
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
