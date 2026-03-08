# Scraping and Scheduling

The core of `rust-yscraper` is its ability to automatically fetch and parse Hacker News threads and keep them updated over time.

## Scraping Logic

The scraper is specialized for Hacker News "Ask HN: What Are You Working On" threads.

### Thread Validation
To ensure data quality, the scraper performs a validation check on the thread title. It must start with the prefix **"Ask HN: What Are You Working On"** (case-insensitive). If the title does not match, the scrape operation will fail with an error.

### Metadata Extraction
The scraper extracts the **Month** and **Year** from the thread title using a regular expression (e.g., "January 2024"). This metadata is used to label and organize the threads in the system.

### Comment Parsing
The scraper targets only **top-level (root) comments**. It identifies them by looking for the indentation spacer image with a width of `0`. For each root comment, it extracts:
- **Comment ID**: The unique identifier from Hacker News.
- **Author**: The username of the commenter.
- **Date**: The timestamp of the comment (extracted from the `title` attribute of the age span).
- **Text**: The full body text of the comment.
- **Subcomment Count**: The number of replies to that comment (if available).

### Persistence (Upsert)
Comments are stored in a PostgreSQL database using "upsert" logic. If a comment with the same ID already exists, its content and metadata are updated. This ensures that edits to comments are captured and duplicates are avoided.

## Background Scheduling

The system includes a background scheduler to keep tracked threads up-to-date.

### Scheduling Parameters
When a link is added, it is assigned two scheduling parameters:
- **Frequency Hours**: How often the thread should be re-scraped (default: 24 hours).
- **Days Limit**: How many days the system should continue to refresh the thread after it was added (default: 7 days).

### Scheduler Operation
A background process runs at a configurable interval (e.g., every minute) and performs the following:
1. Queries the database for all tracked links that are **due for a refresh** (current time > last scraped + frequency) and are still within their **days limit**.
2. For each due link, it adds a new scrape task to the **Task Queue**.

### Task Queue and Deduplication
The system uses a task queue to manage active scraping operations. This ensures that:
- Scraping happens asynchronously and does not block the API.
- Multiple scrapes for the same link are not performed concurrently (deduplication).
- The system can handle multiple threads being added or refreshed at the same time without overloading the network.
