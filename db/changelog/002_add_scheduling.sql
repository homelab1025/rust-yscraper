-- Add scheduling metadata to urls table for comment refreshing
ALTER TABLE urls ADD COLUMN IF NOT EXISTS last_scraped TIMESTAMPTZ;
ALTER TABLE urls ADD COLUMN IF NOT EXISTS frequency_hours INTEGER DEFAULT 24;
ALTER TABLE urls ADD COLUMN IF NOT EXISTS days_limit INTEGER DEFAULT 7;

-- Add indexes for scheduling queries
CREATE INDEX IF NOT EXISTS idx_urls_last_scraped ON urls (last_scraped);
CREATE INDEX IF NOT EXISTS idx_urls_scheduling ON urls (last_scraped, days_limit) WHERE last_scraped IS NOT NULL;