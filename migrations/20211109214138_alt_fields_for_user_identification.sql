-- Add migration script here
ALTER TABLE users ADD COLUMN alt_id_fields jsonb;
