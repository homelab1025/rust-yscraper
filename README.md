# Rust Hacker News Scraper

A Rust-based web application that scrapes comments (only the first layer, not the children of the comments) from Hacker News threads, stores them in a PostgreSQL database, and provides a RESTful API to access the data.

## Overview

This project consists of a web server that:
- Scrapes Hacker News (HN) comment threads.
- Extracts comment text, author, and metadata.
- Persists data in a PostgreSQL database using SQLx.
- Exposes a REST API via Axum.
- Provides a Swagger UI for API exploration.

## Features

- **Concurrent Scraping**: Uses a task queue to handle multiple scrape requests efficiently.
- **REST API**: Endpoints for listing comments, links, and triggering scrapes.
- **Swagger Documentation**: Interactive API documentation available at `/swagger-ui`.
- **Database Persistence**: Robust storage with PostgreSQL and Liquibase for migrations.
- **Containerized**: Easy to deploy and run using K8s.

## Tools Used

- for k8s deployment manifest files Kustomize
- for test coverage cargo llvm-cov
```bash
cargo llvm-cov --html
```