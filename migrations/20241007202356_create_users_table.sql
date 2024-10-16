CREATE TABLE users (
    user_id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL,
    displayname TEXT NOT NULL,
    joined TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    email TEXT NOT NULL,

    -- SHA-256(pass + salt + pepper)
    hash BYTEA NOT NULL,
    salt TEXT NOT NULL
);
