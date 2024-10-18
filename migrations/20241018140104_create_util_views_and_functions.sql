CREATE OR REPLACE FUNCTION is_not_null(user_id_input BIGINT)
RETURNS TABLE (
    condition BOOLEAN
) AS $$
SELECT
    CASE 
        WHEN user_id_input IS NOT NULL THEN true 
        ELSE false
    END AS condition;
$$ LANGUAGE SQL;

CREATE OR REPLACE VIEW get_posts AS
SELECT
    posts.*,
    users.id AS user_id, users.handle, users.displayname
FROM
    posts
    JOIN users ON posts.poster_id = users.id;
