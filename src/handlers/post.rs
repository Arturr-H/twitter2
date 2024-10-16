/* Imports */
use actix_web::{get, post, route, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use crate::{error::Error, middleware::auth::UserClaims, models::{post::{Post, PostBoolean}, post_citation::PostCitation, user::{User, UserInfo}}, utils::logger::log, AppData};

/* Structs */
#[derive(Deserialize)]
struct PublishRequest {
    content: String,
    replies_to: Option<i64>,
    citation: Option<PostCitation>
}
#[derive(Deserialize)]
struct SetBooleanRequest {
    to: bool,
    post_id: i32
}
#[derive(Serialize, FromRow, sqlx::Type)]
pub struct PostWithUser {
    pub post: Post,
    pub user: UserInfo,
    pub liked: bool,
    pub bookmarked: bool,
}


/// Publish a new post
#[post("/publish")]
pub async fn publish(
    req: HttpRequest, data: web::Data<AppData>,
    body: web::Json<PublishRequest>, user: User
) -> impl Responder {
    let body = body.into_inner();
    Post::new(user.user_id(), body.content, body.replies_to, body.citation)
        .insert_into(&data.db)
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
    Post::set_boolean(PostBoolean::Like, &data.db, body.to, user.user_id(), body.post_id.clone())
        .await
        .map(|_| HttpResponse::Ok())
}

/// Set a post to bookmarked or not for user
#[post("/set-bookmark")]
pub async fn set_bookmark(
    req: HttpRequest, data: web::Data<AppData>,
    body: web::Json<SetBooleanRequest>, user: User
) -> impl Responder {
    Post::set_boolean(PostBoolean::Bookmark, &data.db, body.to, user.user_id(), body.post_id.clone())
        .await
        .map(|_| HttpResponse::Ok())
}

/// Get the feed (tied to user token)
#[get("/feed")]
pub async fn feed(
    req: HttpRequest, data: web::Data<AppData>,
    user: User
) -> impl Responder {
    log("feed", "Retrieving feed");

    // ORDER BY 
    // p.created_at DESC;
    sqlx::query_as!(PostWithUser, r#"
        SELECT
            (posts.id, posts.content, posts.total_likes, posts.total_replies, posts.poster_id, posts.replies_to, posts.citation, posts.created_at) AS "post!: Post",
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
                ON posts.id = bookmarks.post_id AND bookmarks.user_id = $1;
    "#, user.user_id())
        .fetch_all(&data.db)
        .await
        .map_err(Error::new)
        .map(|e|
            serde_json::to_string(&e).unwrap()
        )
}

/// Get a specific post by ID
#[get("/id/{id}")]
pub async fn post_by_id(
    req: HttpRequest, data: web::Data<AppData>,
    user: User, path: web::Path<i32>
) -> impl Responder {
    sqlx::query_as!(PostWithUser, r#"
        SELECT
            (posts.id, posts.content, posts.total_likes, posts.total_replies, posts.poster_id, posts.replies_to, posts.citation, posts.created_at) AS "post!: Post",
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
            WHERE posts.id = $2;
    "#, user.user_id(), path.into_inner())
    .fetch_one(&data.db)
    .await
    .map_err(Error::new)
    .map(|e|
        serde_json::to_string(&e).unwrap()
    )
}

// /// Reference a part of a post.
// /// Example:
// /// "Minim quis deserunt culpa ex nulla ex ipsum culpa culpa aliquip"
// /// reference 11-19 => ... "deserunt" ...
// #[get("/reference/{post_id}/{start}-{end}")]
// pub async fn reference(
//     path: web::Path<(i32, i32, i32)>,
//     data: web::Data<AppData>
// ) -> impl Responder {
//     let (post_id, start_ref, end_ref) = path.into_inner();

//     if start_ref >= end_ref {
//         return Err(Error::new("Invalid reference range"));
//     }

//     let post = sqlx::query_as!(Post, r#"
//         SELECT * FROM posts WHERE posts.id = $1;
//     "#, post_id)
//         .fetch_one(&data.db).await
//         .map_err(Error::new)?;

//     let content = post.content;
//     if end_ref > (content.len() - 1) as i32 {
//         return Err(Error::new("Invalid reference (out of range)"));
//     }

//     PostCitation::new(content, start_ref, end_ref)
//         .to_json()
//         .ok_or(Error::new("Cant serialize post citation JSON"))
//         .map(|e| HttpResponse::Ok().json(e))
// }
