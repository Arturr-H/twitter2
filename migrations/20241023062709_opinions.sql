CREATE TABLE post_opinions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    post_id BIGINT REFERENCES posts(id) ON DELETE CASCADE NOT NULL,

    votes INT NOT NULL DEFAULT 0,
    opinion VARCHAR(12) NOT NULL,

    UNIQUE(opinion, post_id)
);

CREATE TABLE post_opinion_votes (
    opinion_id BIGINT REFERENCES post_opinions(id) ON DELETE CASCADE,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    post_id BIGINT REFERENCES posts(id) ON DELETE CASCADE,

    PRIMARY KEY (user_id, post_id, opinion_id)
);

-- Change get_posts_default function to also return
-- some opinions
CREATE OR REPLACE FUNCTION get_posts_default(user_id_input BIGINT)
RETURNS TABLE (
    id BIGINT, content TEXT, created_at TIMESTAMPTZ,
    total_likes BIGINT, total_replies BIGINT,
    poster_id BIGINT,
    replies_to BIGINT,
    citation JSONB,
    
    user_id BIGINT, displayname TEXT, handle TEXT,
    liked BOOLEAN, bookmarked BOOLEAN
) AS $$
    SELECT
        posts.*,
        users.id AS user_id, users.displayname, users.handle,
        is_not_null(post_likes.user_id) AS liked,
        is_not_null(post_bookmarks.user_id) AS bookmarked
    FROM
        posts
        JOIN users ON posts.poster_id = users.id
        LEFT JOIN post_likes     ON post_likes.post_id     = posts.id AND post_likes.user_id     = user_id_input
        LEFT JOIN post_bookmarks ON post_bookmarks.post_id = posts.id AND post_bookmarks.user_id = user_id_input;
$$ LANGUAGE sql;
