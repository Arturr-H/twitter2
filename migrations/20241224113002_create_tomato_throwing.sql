ALTER TABLE posts
ADD COLUMN total_tomatoes BIGINT NOT NULL DEFAULT 0;

ALTER TABLE users
ADD COLUMN tomatoes INT NOT NULL DEFAULT 5,
ADD COLUMN last_tomato_reset TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE TABLE post_tomatoes (
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    post_id BIGINT REFERENCES posts(id) ON DELETE CASCADE NOT NULL,
    thrown_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    
    PRIMARY KEY (user_id, post_id)
);

DROP FUNCTION IF EXISTS get_posts_default(BIGINT);

-- Add migration script here
CREATE OR REPLACE FUNCTION get_posts_default(user_id_input BIGINT)
RETURNS TABLE (
    id BIGINT, content TEXT, created_at TIMESTAMPTZ,
    total_likes BIGINT, total_replies BIGINT,
    poster_id BIGINT,
    replies_to BIGINT,
    citation JSONB,
    total_tomatoes BIGINT,
    
    user_id BIGINT, displayname TEXT, handle TEXT,
    liked BOOLEAN, bookmarked BOOLEAN, has_thrown_tomato BOOLEAN, is_followed BOOLEAN,
    top_opinions JSONB
) AS $$
    WITH ranked_opinions AS (
        SELECT
            po.post_id,
            po.id AS opinion_id,
            po.opinion,
            po.votes,
            EXISTS (
                SELECT 1
                FROM post_opinion_votes pov
                WHERE pov.opinion_id = po.id AND pov.user_id = user_id_input
            ) AS voted,
            ROW_NUMBER() OVER (PARTITION BY po.post_id ORDER BY po.votes DESC) AS rank
        FROM post_opinions po
    ), top_opinions_cte AS (
        SELECT
            post_id,
            jsonb_agg(jsonb_build_object(
                'opinion', opinion,
                'votes', votes,
                'voted', voted,
                'opinion_id', opinion_id
            )) AS opinions
        FROM ranked_opinions
        WHERE rank <= 5
        GROUP BY post_id
    )
    SELECT
        posts.*,
        users.id AS user_id, users.displayname, users.handle,
        is_not_null(post_likes.user_id) AS liked,
        is_not_null(post_bookmarks.user_id) AS bookmarked,
        is_not_null(post_tomatoes.user_id) AS has_thrown_tomato,
        is_not_null(follows.follower_id) AS is_followed,
        COALESCE((
            SELECT opinions
            FROM top_opinions_cte top_op
            WHERE top_op.post_id = posts.id
        ), '[]') AS top_opinions
    FROM
        posts
        JOIN users ON posts.poster_id = users.id
        LEFT JOIN post_likes     ON post_likes.post_id     = posts.id AND post_likes.user_id     = user_id_input
        LEFT JOIN post_bookmarks ON post_bookmarks.post_id = posts.id AND post_bookmarks.user_id = user_id_input
        LEFT JOIN post_tomatoes  ON post_tomatoes.post_id  = posts.id AND post_tomatoes.user_id  = user_id_input
        LEFT JOIN follows        ON follows.follower_id    = user_id_input AND follows.followee_id = posts.poster_id;
$$ LANGUAGE sql;
