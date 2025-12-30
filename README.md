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

## Deployment

### Prerequisites

- **Rust** (latest stable)
- **Node.js** (v24+ recommended) and **npm**
- **Docker** and **Docker Compose**
- **kubectl** and **Kustomize** (for K8s deployment)

### Local Development

1.  **Database Setup**:
    Start the PostgreSQL database and apply migrations using Docker Compose:
    ```bash
    docker compose up -d postgres
    docker compose up --no-deps liquibase
    ```

2.  **Backend**:
    Run the Rust web server:
    ```bash
    cargo run -p web_server
    ```
    The API will be available at `http://localhost:3000` and Swagger UI at `http://localhost:3000/swagger-ui`.

3.  **Frontend**:
    Navigate to the `webapp` directory, install dependencies, and start the development server:
    ```bash
    cd webapp
    npm install
    npm run dev
    ```
    The web application will be accessible at `http://localhost:5173`.

### Docker Compose

For local testing with all dependencies, you can use the provided `docker-compose.yml`. Note that currently, it is primarily configured to run the infrastructure (Postgres, Liquibase, Adminer).

To start the infrastructure:
```bash
docker compose up -d
```

### Kubernetes (Production)

The project includes K8s manifests managed with Kustomize in the `k8s/` directory.

1.  **Build and Push Images**:
    Ensure the Docker images for `web_server` and `webapp` are built and pushed to your registry (the default in `k8s/overlays/prod` is `ghcr.io/homelab1025/rust-yscraper`).

2.  **Configuration**:
    - Update `k8s/overlays/prod/config-prod.toml` with your production settings.
    - Ensure a Kubernetes secret `yscraper-db-password` exists with the necessary database credentials (`username`, `password`, `host`, `port`, `name`) in the namespace where you will deploy the application.

3.  **Deploy**:
    Apply the production overlay:
    ```bash
    kubectl apply -k k8s/overlays/prod
    ```