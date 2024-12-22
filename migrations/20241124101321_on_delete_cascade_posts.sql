ALTER TABLE posts
DROP CONSTRAINT IF EXISTS posts_replies_to_fkey;

ALTER TABLE posts
ADD CONSTRAINT posts_replies_to_fkey
FOREIGN KEY (replies_to)
REFERENCES posts(id)
ON DELETE CASCADE;
