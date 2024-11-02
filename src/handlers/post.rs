/* Imports */
use actix_web::{get, post, route, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use crate::{error::Error, middleware::auth::UserClaims, models::{post::{Post, PostBoolean, PostWithUser}, post_citation::PostCitation, user::{User, UserInfo}}, utils::logger::log, AppData};

/* Structs */
#[derive(Deserialize)]
struct PublishRequest {
    content: String,
    replies_to: Option<i64>,
    citation: Option<PostCitation>
}
#[derive(Deserialize)]
struct DeleteRequest {
    post_id: i64
}
#[derive(Deserialize)]
struct SetBooleanRequest {
    to: bool,
    post_id: i32
}

/// Publish a new post
#[post("/publish")]
pub async fn publish(
    req: HttpRequest, data: web::Data<AppData>,
    body: web::Json<PublishRequest>, user: User
) -> impl Responder {
    let body = body.into_inner();
    Post::new(user.id(), body.content, body.replies_to, body.citation)
        .insert_into(&data.db)
        .await
        .map_err(Error::new)
        .map(|_| HttpResponse::Ok())
}

/// Delete a post
#[post("/delete")]
pub async fn delete(
    req: HttpRequest, data: web::Data<AppData>,
    body: web::Json<DeleteRequest>, user: User
) -> impl Responder {
    sqlx::query!(r#"
        DELETE FROM posts WHERE id = $1 AND poster_id = $2;
    "#, body.post_id, user.id())
        .execute(&data.db)
        .await
        .map_err(Error::new)
        .map(|_| HttpResponse::Ok())
}

/// Set a post to liked or not
#[post("/set-like")]
pub async fn set_like(
    req: HttpRequest, data: web::Data<AppData>,
    body: web::Json<SetBooleanRequest>, user: User
) -> impl Responder {
    Post::set_boolean(PostBoolean::Like, &data.db, body.to, user.id(), body.post_id.clone())
        .await
        .map(|_| HttpResponse::Ok())
}

/// Set a post to bookmarked or not for user
#[post("/set-bookmark")]
pub async fn set_bookmark(
    req: HttpRequest, data: web::Data<AppData>,
    body: web::Json<SetBooleanRequest>, user: User
) -> impl Responder {
    Post::set_boolean(PostBoolean::Bookmark, &data.db, body.to, user.id(), body.post_id.clone())
        .await
        .map(|_| HttpResponse::Ok())
}

/// Get a specific post by ID
#[get("/id/{id}")]
pub async fn post_by_id(
    req: HttpRequest, data: web::Data<AppData>,
    user: User, path: web::Path<i64>
) -> impl Responder {
    sqlx::query_as!(PostWithUser, r#"
        SELECT * FROM get_posts_default($1)
            WHERE id = $2 LIMIT 1;
    "#, user.id(), path.into_inner())
    .fetch_one(&data.db)
    .await
    .map_err(Error::new)
    .map(|e|
        serde_json::to_string(&e).unwrap()
    )
}
