/* Imports */
use actix_web::{get, web, HttpRequest, Responder};
use crate::{error::Error, models::{post::PostWithUser, user::{User, UserIdReq}}, AppData};

/// Get the most recent posts
#[get("/newest")]
pub async fn newest(
    req: HttpRequest, data: web::Data<AppData>,
    user_id: UserIdReq
) -> impl Responder {
    sqlx::query_as!(PostWithUser, r#"
        SELECT * FROM get_posts_default($1)
            WHERE (replies_to IS NULL
                OR citation IS NOT NULL)
            ORDER BY created_at DESC;
    "#, user_id.0)
        .fetch_all(&data.db).await
        .map_err(Error::new)
        .map(|e| serde_json::to_string(&e).unwrap())
}

/// Get the most popular posts this week
#[get("/popular")]
pub async fn popular(
    req: HttpRequest, data: web::Data<AppData>,
    user_id: UserIdReq
) -> impl Responder {
    sqlx::query_as!(PostWithUser, r#"
        SELECT * FROM get_posts_default($1)
            WHERE (replies_to IS NULL
                OR citation IS NOT NULL)
                AND created_at > now() - interval '7 days'
            ORDER BY total_likes DESC;
    "#, user_id.0)
        .fetch_all(&data.db).await
        .map_err(Error::new)
        .map(|e| serde_json::to_string(&e).unwrap())
}

/// I might change this in the future but currently
/// this will return some post from users that the 
/// requesting user is following and some posts that
/// these people have liked.
#[get("/for-you")]
pub async fn for_you(
    req: HttpRequest, data: web::Data<AppData>,
    user_id: UserIdReq
) -> impl Responder {
    dbg!(user_id.0);
    
    sqlx::query_as!(PostWithUser, r#"
        SELECT * FROM get_posts_default($1) posts
            WHERE posts.poster_id IN (
                SELECT follows.followee_id FROM follows
                    WHERE follows.follower_id = $1
            )
            OR posts.id IN (
                SELECT post_id FROM post_likes
                    WHERE user_id IN (
                        SELECT follows.followee_id FROM follows
                            WHERE follows.follower_id = $1
                    )
            )
            
            AND (posts.replies_to IS NULL
                OR posts.citation IS NOT NULL)
            ORDER BY posts.created_at DESC;
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
