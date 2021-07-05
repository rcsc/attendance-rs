-- Add migration script here
ALTER TABLE tokens ALTER COLUMN expiration_time SET NOT NULL;
