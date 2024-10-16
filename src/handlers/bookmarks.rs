/* Imports */
use actix_web::{get, web, Responder};
use crate::{error::Error, models::user::User, AppData, models::{post::{Post, PostWithUser}, user::UserInfo}};

#[get("/bookmarks")]
pub async fn bookmarks(
    data: web::Data<AppData>,
    user: User
) -> impl Responder {
    sqlx::query_as!(PostWithUser, r#"
        SELECT
            (posts.*, NULL) AS "post!: Post",
            (users.user_id, users.handle, users.displayname) AS "user!: UserInfo",

            -- Add boolean for if liked or bookmarked
            COALESCE(likes.user_id IS NOT NULL, false) AS "liked!: bool",
            COALESCE(bookmarks.user_id IS NOT NULL, false) AS "bookmarked!: bool"
        FROM
            posts
            JOIN users ON posts.poster_id = users.user_id
            LEFT JOIN likes
                ON posts.id = likes.post_id AND likes.user_id = $1
            LEFT JOIN bookmarks
                ON posts.id = bookmarks.post_id AND bookmarks.user_id = $1
            JOIN bookmarks bm ON posts.id = bm.post_id
        WHERE
            bm.user_id = $1;
    "#, user.user_id())
        .fetch_all(&data.db)
        .await
        .map_err(Error::new)
        .map(|e|
            serde_json::to_string(&e).unwrap()
        )
}
