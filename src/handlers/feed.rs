/* Imports */
use actix_web::{get, web, HttpRequest, Responder};
use crate::{error::Error, models::{post::PostWithUser, user::{User, UserIdReq}}, utils::logger::log, AppData};

/// Get the most recent posts
#[get("/newest")]
pub async fn newest(
    req: HttpRequest, data: web::Data<AppData>,
    user_id: UserIdReq
) -> impl Responder {
    log("feed", format!("Retrieving feed for user(id = {})", user_id.0));

    sqlx::query_as!(PostWithUser, r#"
        SELECT * FROM get_posts_default($1)
            WHERE replies_to IS NULL
                OR citation IS NOT NULL
            ORDER BY created_at DESC;
    "#, user_id.0)
        .fetch_all(&data.db).await
        .map_err(Error::new)
        .map(|e| serde_json::to_string(&e).unwrap())
}

/// Get replies for a specific post
#[get("/replies/{post_id}")]
pub async fn replies(
    req: HttpRequest, data: web::Data<AppData>,
    user_id: UserIdReq, path: web::Path<i64>
) -> impl Responder {
    sqlx::query_as!(PostWithUser, r#"
        SELECT posts.* FROM get_posts_default($1) posts
            WHERE posts.replies_to = $2
            ORDER BY total_likes DESC;
    "#, user_id.0, path.into_inner())
        .fetch_all(&data.db).await
        .map_err(Error::new)
        .map(|e| serde_json::to_string(&e).unwrap())
}

/// Search for content
#[get("/search/{query}")]
pub async fn search(
    req: HttpRequest, data: web::Data<AppData>,
    user_id: UserIdReq, query: web::Path<String>
) -> impl Responder {
    let array: Vec<String> = serde_json::from_str::<Vec<String>>(&query.into_inner())
        .map_err(Error::new)?
        .into_iter()
        .map(|e| format!("%{e}%"))
        .collect();
dbg!(&array);
    /* No search */
    if array.is_empty() { return Ok(String::from("[]")) }

    sqlx::query_as!(PostWithUser, r#"
        SELECT posts.* FROM get_posts_default($1) posts
            WHERE posts.content ILIKE ALL($2)
            ORDER BY total_likes DESC;
    "#, user_id.0, &array)
        .fetch_all(&data.db).await
        .map_err(Error::new)
        .map(|e| serde_json::to_string(&e).unwrap())
}
