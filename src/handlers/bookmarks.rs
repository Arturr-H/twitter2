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
            posts.*,
            is_not_null(post_likes.user_id) AS liked,
            is_not_null(post_bookmarks.user_id) AS bookmarked
        FROM
            get_posts posts
            LEFT JOIN post_likes     ON post_likes.post_id     = posts.id AND post_likes.user_id     = $1
            LEFT JOIN post_bookmarks ON post_bookmarks.post_id = posts.id AND post_bookmarks.user_id = $1
            
            JOIN post_bookmarks bm ON posts.id = bm.post_id
        WHERE
            bm.user_id = $1;
    "#, user.id())
        .fetch_all(&data.db)
        .await
        .map_err(Error::new)
        .map(|e|
            serde_json::to_string(&e).unwrap()
        )
}
