-- Add migration script here
ALTER TABLE users
RENAME COLUMN username TO handle;
