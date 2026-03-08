# Comment Management

The core of the `rust-yscraper` experience is the ability to efficiently triage and manage comments from tracked threads.

## Comment States (Triage)
Each comment in the system is assigned a state. The primary purpose of the triage process is to move comments from their initial "New" state to a final state (either "Picked" or "Discarded").

The possible comment states are:
- **New (Default)**: The initial state of a comment when first scraped.
- **Picked**: A comment that the user has reviewed and deemed interesting or worthy of archiving.
- **Discarded**: A comment that the user has reviewed and decided not to keep.

## Viewing and Filtering
The system provides a dedicated interface for viewing comments for a specific link. Users can filter the list by their current state to focus on the triage process.

### Sorting
Comments can be sorted by:
- **Date**: The chronological order of the comment (default: Descending, showing the newest first).
- **Subcomment Count**: The number of replies a comment received. This is useful for finding the most active or discussed updates in a thread.

### Pagination
The comment list is paginated (default: 50 items per page) to ensure fast loading and easy navigation of large threads.

## Functional Requirements
- When a comment's state is updated, the change must be persisted in the database.
- If a user is viewing a filtered list (e.g., "New" comments), updating a comment's state to "Picked" or "Discarded" should immediately remove it from the current view.
- The UI must provide clear visual feedback to indicate the current state of a comment.
- The triage process must be optimized for speed, supporting keyboard-driven actions.
