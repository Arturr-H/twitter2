-- Users
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    handle VARCHAR(25) UNIQUE NOT NULL,
    displayname VARCHAR(50) NOT NULL,
    joined TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    email TEXT UNIQUE NOT NULL,

    -- SHA-256(pass + salt + pepper)
    hash BYTEA NOT NULL,
    salt TEXT NOT NULL
);

-- Posts
CREATE TABLE posts (
    id BIGSERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,

    -- All individual likes are stored in a separate table, because
    -- we don't want to have to count every single table each time
    -- we want to view the likes.
    total_likes BIGINT DEFAULT 0 NOT NULL,
    total_replies BIGINT DEFAULT 0 NOT NULL,

    -- The Serial primary key of the poster
    poster_id BIGINT NOT NULL REFERENCES users(id),
    replies_to BIGINT REFERENCES posts(id),

    -- Quoting another post (PostCitation struct)
    -- Can be null even if replies_to isn't
    citation JSONB
);

-- Hashtags
CREATE TABLE hashtags (
    id BIGSERIAL PRIMARY KEY,
    tag VARCHAR(255) UNIQUE NOT NULL
);

-- POST => LIKE
CREATE TABLE post_likes (
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    post_id BIGINT REFERENCES posts(id) ON DELETE CASCADE NOT NULL,

    -- Only one like per post per user
    PRIMARY KEY (user_id, post_id)
);

-- POST => BOOKMARK
CREATE TABLE post_bookmarks (
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    post_id BIGINT REFERENCES posts(id) ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (user_id, post_id)
);

-- POST => HASHTAG (contained in post(content) column)
CREATE TABLE post_hashtags (
    post_id BIGINT REFERENCES posts(id) ON DELETE CASCADE NOT NULL,
    hashtag_id BIGINT REFERENCES hashtags(id) ON DELETE CASCADE NOT NULL,

    -- Only one reference to the same hashtag per post_id
    PRIMARY KEY (post_id, hashtag_id)
);
