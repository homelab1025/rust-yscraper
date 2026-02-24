# Project Overview: rust-yscraper

## Purpose
A web server designed for scraping tasks, specifically targeting URL and comment data (e.g., from Hacker News). It uses a task deduplication queue to manage scraping work efficiently.

## Tech Stack
- **Languages**: Rust (Backend), TypeScript (Frontend / API Clients).
- **Web Framework**: Axum.
- **Database**: Postgres (managed with Liquibase migrations).
- **Concurrency**: Tokio (async/await).
- **Dependencies**: cargo, docker-compose, npm.
