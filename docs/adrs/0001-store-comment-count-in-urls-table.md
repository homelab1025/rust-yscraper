# ADR 0001: Store Comment Count in Urls Table

## Status
Accepted

## Context
The application needs to display the number of comments for each link in the dashboard. Calculating this on every page load would require a join or a count query against the `comments` table, which could become slow as the number of comments grows.

## Decision
We have added `comment_count` and `picked_comment_count` columns (unsigned `u32`) to the `urls` table. These columns are updated:
1.  Initially during the migration process for existing records.
2.  Automatically in the application code after each successful scrape task completes.
3.  Automatically after a user updates a comment's state.

### Architectural Rules
> [!IMPORTANT]
> The `comment_count` and `picked_comment_count` metrics MUST only be used for presentation purposes (e.g., in the link management table). They should NOT be used as a source of truth for application logic that requires high consistency with the `comments` table.

## Consequences
- **Pros**: 
  - Faster UI rendering for the link management page.
  - Simplified API for listing links (no need for complex joins).
- **Cons**:
  - Slight redundancy in the data.
  - Requires manual updates after scraping to ensure consistency.
