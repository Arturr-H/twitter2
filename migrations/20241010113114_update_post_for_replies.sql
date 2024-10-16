ALTER TABLE posts
ADD COLUMN replies_to BIGINT,
ADD CONSTRAINT fk_replies_to
    FOREIGN KEY (replies_to)
    REFERENCES posts(id);
