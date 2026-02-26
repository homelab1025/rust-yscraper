# Implementation Plan - Manual Scraping Trigger

## Phase 1: Logic & Feedback UI
- [ ] Task: Implement manual scrape trigger logic in `LinkManagementPage.tsx`
    - [ ] Add `handleManualScrape` function that calls `linksApi.scrapeLink`.
    - [ ] Handle `Scheduled` and `AlreadyScheduled` responses from the backend.
    - [ ] Implement error handling for the API call.
- [ ] Task: Implement Snackbar notification UI in `LinkManagementPage.tsx`
    - [ ] Add MUI `Snackbar` and `Alert` components.
    - [ ] Create state to manage the snackbar's message, visibility, and severity.
    - [ ] Connect the snackbar to the `handleManualScrape` results.
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: UI Integration & Final Polish
- [ ] Task: Add Manual Scrape button to the Links Table
    - [ ] Import `RefreshIcon` from `@mui/icons-material`.
    - [ ] Add an `IconButton` to the "Actions" column in the links table row.
    - [ ] Connect the icon button to the `handleManualScrape` function.
- [ ] Task: Final end-to-end verification
    - [ ] Manually verify the full flow: click button -> observe snackbar message -> verify backend log output (if accessible).
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)
