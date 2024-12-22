-- The user_id_input is the id of the user who
-- is retrieving posts (who is viewing the posts)
-- and is used to determine wether booleans like
-- "bookmarked" or "liked" should be true.
CREATE OR REPLACE FUNCTION get_posts_default(user_id_input BIGINT)
RETURNS TABLE (
    id BIGINT, content TEXT, created_at TIMESTAMPTZ,
    total_likes BIGINT, total_replies BIGINT,
    poster_id BIGINT,
    replies_to BIGINT,
    citation JSONB,
    
    user_id BIGINT, displayname TEXT, handle TEXT,
    liked BOOLEAN, bookmarked BOOLEAN, is_followed BOOLEAN
) AS $$
    SELECT
        posts.*,
        users.id AS user_id, users.displayname, users.handle,
        is_not_null(post_likes.user_id) AS liked,
        is_not_null(post_bookmarks.user_id) AS bookmarked,
        is_not_null(follows.follower_id) AS is_followed
    FROM
        posts
        JOIN users ON posts.poster_id = users.id
        LEFT JOIN post_likes     ON post_likes.post_id     = posts.id AND post_likes.user_id     = user_id_input
        LEFT JOIN post_bookmarks ON post_bookmarks.post_id = posts.id AND post_bookmarks.user_id = user_id_input
        LEFT JOIN follows        ON follows.follower_id    = user_id_input AND follows.followee_id = posts.poster_id;
$$ LANGUAGE sql;
