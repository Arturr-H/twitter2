//! This file contains routes for dealing with
//! hashtags, e.g retrieving posts which have a
//! certain hashtag, or getting trending hashtags
//! etc.

/* Imports */
use actix_web::{get, web, Responder};
use serde::Serialize;
use crate::{error::Error, models::{post::Post, user::{User, UserInfo}}, AppData, models::post::PostWithUser};

/* Structs */
#[derive(Serialize)]
struct TrendingHashtag {
    tag: String,
    usage_count: i64,
}

#[get("/single/{hashtag}")]
pub async fn posts_by_hashtag(
    path: web::Path<String>,
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
            
            JOIN post_hashtags ph ON posts.id = ph.post_id
            JOIN hashtags h ON ph.hashtag_id = h.id
        WHERE
            h.tag = $2;
    "#, user.id(), path.into_inner())
        .fetch_all(&data.db)
        .await
        .map_err(Error::new)
        .map(|e|
            serde_json::to_string(&e).unwrap()
        )
}

#[get("/trending-today")]
pub async fn trending_hashtags(
    data: web::Data<AppData>,
) -> impl Responder {
    sqlx::query!(r#"
        SELECT h.tag, COUNT(*) as usage_count
        FROM post_hashtags ph
        JOIN hashtags h ON h.id = ph.hashtag_id
        JOIN posts p ON p.id = ph.post_id
        WHERE p.created_at >= NOW() - INTERVAL '24 hours'
        GROUP BY h.tag
        ORDER BY usage_count DESC
        LIMIT 10;
    "#)
    .fetch_all(&data.db)
    .await
    .map_err(Error::new)
    .and_then(|result|
        serde_json::to_string(&result
            .into_iter()
            .map(|e| TrendingHashtag {
                tag: e.tag,
                usage_count: e.usage_count.unwrap_or(0)
            })
            .collect::<Vec<TrendingHashtag>>()
        ).map_err(Error::new)
    )
}
