-- Liquibase formatted SQL optional; plain SQL is fine as referenced by YAML

-- Create urls table
CREATE TABLE IF NOT EXISTS urls (
    id BIGINT PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    date_added TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create comments table
CREATE TABLE IF NOT EXISTS comments (
    id BIGINT PRIMARY KEY,
    author TEXT NOT NULL,
    date TEXT NOT NULL,
    text TEXT NOT NULL,
    url_id BIGINT NOT NULL REFERENCES urls(id)
);

-- Helpful indexes
CREATE INDEX IF NOT EXISTS idx_comments_date_id_desc ON comments (date DESC, id DESC);
