# Link Management

The system allows users to manage a collection of tracked Hacker News threads. Each thread is identified by its unique HN item ID.

## Core Features
- **Add New Link**: Users can input a Hacker News item ID to start tracking a new thread. The system will automatically construct the target URL: `https://news.ycombinator.com/item?id={item_id}`.
- **List Tracked Links**: A management page displays all currently tracked links with their metadata.
- **Delete Link**: Users can stop tracking a thread and delete all its associated comments. This is a cascading delete operation.

## Link Metadata
For each tracked link, the system displays:
- **ID**: The unique database ID for the link.
- **Thread Title (Metadata)**: The system extracts the thread's "Month Year" from its title (if available) and displays it in the list (e.g., "January 2024").
- **Date Added**: The timestamp when the link was first added to the system.
- **Comment Counts**: Displays the number of "Picked" comments versus the "Total" number of comments captured for that link (e.g., "15 / 120").
- **Status**: The current status of the link (e.g., "Scraped").

## Functional Requirements
- When adding a link, the system must validate the input as a valid HN item ID (integer).
- The system must ensure that duplicate item IDs are not added (using "upsert" logic).
- Deleting a link must be confirmed by the user to prevent accidental data loss.
