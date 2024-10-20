/* Imports */
use actix_web::{get, web, Responder};
use crate::{error::Error, models::user::User, AppData, models::{post::{Post, PostWithUser}, user::UserInfo}};

#[get("/bookmarks")]
pub async fn bookmarks(
    data: web::Data<AppData>,
    user: User
) -> impl Responder {
    sqlx::query_as!(PostWithUser, r#"
        SELECT posts.* FROM get_posts_default($1) posts
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
