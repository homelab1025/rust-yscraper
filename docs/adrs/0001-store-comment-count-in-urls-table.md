# ADR 0001: Store Comment Count in Urls Table

## Status
Accepted

## Context
The application needs to display the number of comments for each link in the dashboard. Calculating this on every page load would require a join or a count query against the `comments` table, which could become slow as the number of comments grows.

## Decision
We have added a `comment_count` column (unsigned `u32`) to the `urls` table. This column is updated:
1.  Initially during the migration process for existing records.
2.  Automatically in the application code after each successful scrape task completes.

## Consequences
- **Pros**: 
  - Faster UI rendering for the link management page.
  - Simplified API for listing links (no need for complex joins).
- **Cons**:
  - Slight redundancy in the data.
  - Requires manual updates after scraping to ensure consistency.
