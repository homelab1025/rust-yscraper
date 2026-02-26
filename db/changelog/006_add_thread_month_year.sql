-- Create thread_month and thread_year columns in urls table
ALTER TABLE urls ADD COLUMN thread_month INTEGER;
ALTER TABLE urls ADD COLUMN thread_year INTEGER;
