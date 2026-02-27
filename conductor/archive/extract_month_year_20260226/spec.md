# Specification: Extract Month and Year from Thread Title

## Overview
This track introduces the extraction of month and year from the Hacker News "Ask HN: What Are You Working On" thread titles. This information will be stored in the database and used as the display text for the links in the links management table.

## Functional Requirements
- **Scraper Enhancement**:
    - Update `web_server/src/scrape.rs` to extract the month and year from titles like "Ask HN: What Are You Working On (February 2026)".
    - Use a regex-based approach for extraction.
- **Database Update**:
    - Add `thread_month` (INTEGER) and `thread_year` (INTEGER) columns to the `urls` table via a new Liquibase migration.
    - Update `LinksRepository` and its PostgreSQL implementation to store and retrieve these fields.
    - **Update Policy**: These fields SHOULD be updated upon re-scraping if available. If they were previously populated but the new title does not contain them, they should be cleared (null).
- **API Update**:
    - Update `LinkDto` in `web_server/src/api/links.rs` to include `thread_month` and `thread_year`.
- **Frontend Update**:
    - In `LinkManagementPage.tsx`, the "URL" column will display "Month Year" (e.g., "February 2026") as the link text.
    - The integer month will be mapped to its full name (e.g., 2 -> "February").
    - **Fallback**: If `thread_month` or `thread_year` is null, display the raw URL as the link text.

## Acceptance Criteria
- [ ] Database schema updated with `thread_month` and `thread_year` (both INTEGER).
- [ ] Scraper extracts month and year from thread titles.
- [ ] Links table displays "Month Year" as link text when available.
- [ ] Raw URL is displayed when month/year is unavailable.
- [ ] Re-scraping updates or clears these fields appropriately.

## Out of Scope
- Filtering or sorting by month/year in the UI.
