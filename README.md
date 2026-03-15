# Project Functionality and Functional Analysis

## Project Overview

`rust-yscraper` is a specialized web scraping and monitoring tool designed to track and archive comments from Hacker News (HN). It focuses particularly on the "Ask HN: What Are You Working On" recurring threads, allowing users to keep a historical record of community updates over time.

The project is built with a Rust backend (Axum), a PostgreSQL database, and a React-based web frontend.

## Core Functionality

### 1. Targeted Scraping

The application allows users to trigger a scrape for a specific Hacker News item ID. The scraper fetches the HTML from HN and extracts key information:

- **Comment Metadata**: Author, timestamp, and unique comment ID.
- **Comment Content**: The text body of the comment.
- **Root-Level Extraction**: The scraper specifically targets top-level comments (direct responses to the thread), filtering out nested replies.

### 2. Automated Monitoring (Background Scheduling)

Beyond manual triggers, the system includes a background scheduler that handles periodic refreshes of tracked links:

- **Configurable Frequency**: Users can define how often a thread should be re-scraped (e.g., every 24 hours).
- **Time Limits**: Scrapes can be limited to a specific duration after the link was added (e.g., stop refreshing after 7 days).
- **Task Deduplication**: A task queue ensures that the same link isn't being scraped multiple times concurrently.

### 3. Data Persistence

All extracted data is stored in a PostgreSQL database:

- **Incremental Updates**: The system uses "upsert" logic (insert or update on conflict) to ensure that comment text changes or re-scrapes don't create duplicate records.
- **Historical Tracking**: Stores when links were added and when they were last successfully scraped.

### 4. RESTful API & UI

- **API Endpoints**: Provides endpoints for triggering scrapes, listing stored comments (with pagination), listing tracked links, and deleting links (which cascades to their comments).
- **Swagger Documentation**: Interactive documentation is available for all API endpoints.
- **Web Frontend**: A user interface to view comments, add new threads for tracking, and manage existing links.

---

## Potential Functional Issues & Limitations

### Acceptable ones

- High Specialization (Rigidity): The scraper contains a mandatory validation step that checks if the thread title starts with "Ask HN: What Are You Working On".
- Shallow Scraping (No Nested Comments): The current logic explicitly filters for comments with an indentation level of 0.
- Blindness to Comment Deletions/Moves: The system uses an "upsert" strategy based on the comment ID.

### To consider repairing

#### Fragility to HTML Structure Changes

The scraper relies on specific CSS classes (`tr.athing.comtr`) and, more critically, on the `width` attribute of a spacer image to determine comment depth.

- **Impact**: Hacker News occasionally updates its markup. If they move to a modern CSS-based layout for indentation or change their class names, the scraper will stop functioning immediately and may require significant updates to its selectors.

#### Rate Limiting and IP Reputation

The implementation lacks explicit handling for HTTP 429 (Too Many Requests) or back-off strategies tailored to HN's specific anti-scraping measures.

- **Impact**: Rapidly adding many links or running the scheduler with very high frequency could result in the server's IP being temporarily or permanently blocked by Hacker News.
