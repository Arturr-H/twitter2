-- Create "template" function for getting posts with user info
CREATE VIEW posts_with_user_info AS
    SELECT
        posts.*,
        users.user_id, users.handle, users.displayname
    FROM
        posts
        JOIN users ON posts.poster_id = users.user_id;

