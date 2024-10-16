CREATE TABLE bookmarks (
    user_id BIGINT REFERENCES users(user_id) ON DELETE CASCADE,
    post_id INT REFERENCES posts(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, post_id)
);
