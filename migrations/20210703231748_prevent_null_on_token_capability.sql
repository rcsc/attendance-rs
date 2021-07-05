-- Add migration script here
ALTER TABLE tokens ALTER COLUMN capability SET NOT NULL;
