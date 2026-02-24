-- Add comment_count column to urls table
ALTER TABLE urls ADD COLUMN IF NOT EXISTS comment_count INTEGER DEFAULT 0;

-- Initialize comment_count for existing urls
UPDATE urls u
SET comment_count = (
    SELECT COUNT(*)
    FROM comments c
    WHERE c.url_id = u.id
);
