# System Overview

`rust-yscraper` is a specialized web scraping and monitoring tool designed to track and archive comments from Hacker News (HN). It focuses particularly on the "Ask HN: What Are You Working On" recurring threads, allowing users to keep a historical record of community updates over time. The goal is to select those ideas that seem worth-while to invest time in and so pick up ideas on what to implement next as pet projects or eventually for generating revenue.

## Core Purpose

The tool addresses the need for a structured way to monitor specific, recurring Hacker News threads about interesting ideas of projects. By identifying and extracting top-level comments, it provides a cleaner view of the "What Are You Working On" threads.

## High-Level Architecture

The project follows a standard client-server architecture:

- **Frontend (Web App)**: A React-based application built with TypeScript and Vite. It uses Material UI for the interface and interacts with the backend via a generated REST API client.
- **Backend (Web Server)**: A Rust application using the Axum web framework. It handles API requests, manages the task queue for scraping, and coordinates the background scheduler.
- **Database**: A PostgreSQL database used for persistent storage of tracked links, extracted comments, and scheduling metadata.
- **Scraper**: A specialized Rust module that fetches and parses Hacker News HTML using CSS selectors and specific markup patterns.

## Key Features

- **Tracked Link Management**: Add HN item IDs to a list of tracked threads.
- **Automated Scraping**: Periodically refreshes tracked links to capture new comments.
- **Comment Triage**: A dedicated interface to review new comments and mark them as "Picked" or "Discarded".
- **Keyboard-Centric Workflow**: Optimized for fast triaging using keyboard shortcuts.
- **Historical Archive**: Maintains a record of all captured comments, even if the original thread moves or changes.
