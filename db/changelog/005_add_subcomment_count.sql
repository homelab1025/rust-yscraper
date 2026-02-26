-- liquibase formatted sql
-- changeset flodia:add_subcomment_count

ALTER TABLE comments ADD COLUMN subcomment_count INTEGER DEFAULT 0 NOT NULL;
