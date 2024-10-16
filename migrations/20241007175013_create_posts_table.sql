-- Add migration script here
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL
);