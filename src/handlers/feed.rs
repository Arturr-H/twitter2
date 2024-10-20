/* Imports */
use actix_web::{get, web, HttpRequest, Responder};
use crate::{error::Error, models::{post::PostWithUser, user::User}, utils::logger::log, AppData};

/// Get the most recent posts
#[get("/newest")]
pub async fn newest(
    req: HttpRequest, data: web::Data<AppData>,
    user: User
) -> impl Responder {
    log("feed", format!("Retrieving feed for user(id = {})", user.id()));

    sqlx::query_as!(PostWithUser, r#"
        SELECT * FROM get_posts_default($1)
            ORDER BY created_at DESC;
    "#, user.id())
        .fetch_all(&data.db).await
        .map_err(Error::new)
        .map(|e| serde_json::to_string(&e).unwrap())
}
