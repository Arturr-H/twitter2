ALTER TABLE posts

-- All individual likes are stored in a separate table, because
-- we don't want to have to count every single table each time
-- we want to view the likes.
ADD COLUMN total_likes BIGINT DEFAULT 0 NOT NULL,
ADD COLUMN total_replies BIGINT DEFAULT 0 NOT NULL,

-- The Serial primary key of the poster
ADD COLUMN poster_id BIGINT NOT NULL,
ADD CONSTRAINT fk_poster
    FOREIGN KEY (poster_id)
    REFERENCES users(user_id);
