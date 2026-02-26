# Specification: Manual Scraping Trigger

## Overview
This track introduces a manual scraping trigger for Hacker News links within the link management interface. Users will be able to manually initiate a scrape for a specific link without waiting for the background scheduler to pick it up.

## Functional Requirements
- Add a "Manual Scrape" button to each row in the links table within the `LinkManagementPage`.
- The button will be located in the "Actions" column at the end of the row, alongside existing icons (Comment, Delete).
- The button will use the `RefreshIcon` from `@mui/icons-material`.
- Clicking the button will trigger a `POST /scrape` request to the backend with the corresponding `item_id`.
- The scrape will use the default system settings for `days_limit` (7 days) and `frequency_hours` (24 hours).
- Upon successful scheduling, a Snackbar (toast) notification will appear at the bottom of the screen to confirm the action.
- The button will remain enabled even if a scrape is already in progress (matching the backend's ability to handle "AlreadyScheduled" states).

## UI/UX Requirements
- Confirmation Message: "Scraping scheduled successfully" for `ScrapeState::Scheduled`.
- Alternative Message: "Scraping already in progress for this link" for `ScrapeState::AlreadyScheduled`.
- Error Handling: Show a red Snackbar if the request fails.

## Acceptance Criteria
- [ ] A new button with a "Refresh" icon is present in the "Actions" column for each link.
- [ ] Clicking the button calls the `POST /scrape` endpoint.
- [ ] A Snackbar message "Scraping scheduled successfully" appears on success.
- [ ] A Snackbar message "Scraping already in progress for this link" appears if the backend returns `AlreadyScheduled`.
- [ ] An error Snackbar appears if the request fails for other reasons.

## Out of Scope
- Customizing scraping parameters (days limit, frequency) via the UI during manual trigger.
- Visual indicators of "currently scraping" state on the button itself (beyond the toast feedback).
