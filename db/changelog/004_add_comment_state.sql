-- Add state column to comments table
-- 0: New, 1: Picked, 2: Discarded
ALTER TABLE comments ADD COLUMN IF NOT EXISTS state INTEGER NOT NULL DEFAULT 0;

-- Add picked_comment_count column to urls table
ALTER TABLE urls ADD COLUMN IF NOT EXISTS picked_comment_count INTEGER NOT NULL DEFAULT 0;
