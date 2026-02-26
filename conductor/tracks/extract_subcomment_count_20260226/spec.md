# Specification: Extract and Display Subcomment Count

## Overview
Currently, `rust-yscraper` scrapes Hacker News comments but does not capture the engagement level within the thread itself (i.e., how many subcomments a particular comment has). Hacker News provides this data in the `n` attribute of elements with classes `togg` and `clicky`. This feature aims to extract this value, store it in the database, and display it on the comments page.

## Functional Requirements
1.  **Backend (Scraping):**
    *   Modify the comment scraping logic to locate elements with classes `togg` and `clicky`.
    *   Extract the value of the `n` attribute from these elements.
    *   Default to `0` if the attribute or element is missing.
2.  **Database:**
    *   Add a new column `subcomment_count` (integer, default 0, non-nullable) to the `comments` table.
    *   Update the database schema using Liquibase by adding a new changelog file and updating `changelog-master.yaml`.
3.  **Backend (API):**
    *   Update `CommentsRepository` trait and `PgCommentsRepository` implementation.
    *   Ensure the `subcomment_count` is included in the API response for comments (annotate with `utoipa` for OpenAPI).
    *   Regenerate TypeScript API client after backend changes using the commands specified in `AGENTS.md`.
4.  **Frontend:**
    *   Add a new column "Subcomments" to the comments table on the Comments page.
    *   Position this column after the "Author" column.
    *   Display the value of `subcomment_count` in this column.

## Non-Functional Requirements
*   **Performance:** Scraping the additional attribute should have minimal impact on scraping time.
*   **Accuracy:** The extracted count should precisely match the value on Hacker News.
*   **Code Quality:** Adhere to `AGENTS.md` (naming conventions, no `unwrap`, error handling, `cargo fmt`, `cargo clippy`).

## Acceptance Criteria
*   [ ] Database migration adds `subcomment_count` to the `comments` table.
*   [ ] Scraping a comment with subcomments correctly populates `subcomment_count`.
*   [ ] Scraping a comment with NO subcomments sets `subcomment_count` to `0`.
*   [ ] The API returns the `subcomment_count` for each comment.
*   [ ] TypeScript API client is regenerated and used in the frontend.
*   [ ] The Comments page displays a "Subcomments" column after the "Author" column with the correct values.

## Out of Scope
*   Sorting by subcomment count.
*   Filtering by subcomment count.
*   Updating subcomment count for already scraped comments (unless rescraped).
