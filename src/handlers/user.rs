use actix_multipart::{form::{tempfile::TempFile, MultipartForm}, Multipart};
/* Imports */
use actix_web::{get, post, route, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use crate::{error::Error, middleware::auth::UserClaims, models::{pfp::ProfileImageHandler, post::{Post, PostBoolean, PostWithUser}, post_citation::PostCitation, user::{User, UserIdReq, UserInfo}}, utils::logger::log, AppData};
use image::{self, imageops::resize, EncodableLayout};

/* Structs */
#[derive(Deserialize)]
struct SetFollowingRequest {
    /// The person gaining or losing a follower
    followee_id: i64,
    follow: bool
}

/// Get user by their id
#[get("/id/{id}")]
pub async fn get_by_id(
    data: web::Data<AppData>, _user_id: UserIdReq,
    id: web::Path<i64>
) -> impl Responder {
    let id = id.into_inner();

    sqlx::query_as!(UserInfo, r#"
        SELECT 
            users.id as user_id,
            users.displayname,
            users.handle
        FROM users WHERE users.id = $1;
    "#, id)
    .fetch_optional(&data.db).await
    .map_err(Error::new)
    .and_then(|e| 
        e.ok_or(Error::new("No user found"))
        .and_then(|e| serde_json::to_string(&e)
            .map_err(Error::new))
    )
}
/// Get user by their handle
#[get("/handle/{handle}")]
pub async fn get_by_handle(
    data: web::Data<AppData>, _user_id: UserIdReq,
    handle: web::Path<String>
) -> impl Responder {
    let handle = handle.into_inner();

    sqlx::query_as!(UserInfo, r#"
        SELECT 
            users.id as user_id,
            users.displayname,
            users.handle
        FROM users WHERE users.handle = $1;
    "#, handle)
    .fetch_optional(&data.db).await
    .map_err(Error::new)
    .and_then(|e| 
        e.ok_or(Error::new("No user found"))
        .and_then(|e| serde_json::to_string(&e)
            .map_err(Error::new))
    )
}

/// Get user by their id
#[post("/set-following")]
pub async fn set_following(
    body: web::Json<SetFollowingRequest>,
    data: web::Data<AppData>,
    user_id: UserIdReq
) -> impl Responder {
    User::set_following(
        &data.db,
        user_id.0, body.followee_id,
        body.follow
    ).await
    .map(|_| HttpResponse::Ok())
    .map_err(Error::new)
}

/// Returns info about the user that sends the request
#[get("/profile")]
pub async fn profile(req: HttpRequest, user: User) -> impl Responder {
    serde_json::to_string(&user.to_non_sensitive())
        .map_err(Error::new)
}

/// Get all posts that a user has posted
#[get("/posts/{id}")]
pub async fn posts(
    req: HttpRequest, data: web::Data<AppData>,
    user_id: UserIdReq, path: web::Path<i64>
) -> impl Responder {
    sqlx::query_as!(PostWithUser, r#"
        SELECT posts.* FROM get_posts_default($1) posts
            WHERE posts.poster_id = $2
            ORDER BY created_at DESC;
    "#, user_id.0, path.into_inner())
        .fetch_all(&data.db).await
        .map_err(Error::new)
        .map(|e| serde_json::to_string(&e).unwrap())
}

/// Get profile image of some user
#[get("/profile-image/{id}")]
pub async fn get_profile_image(
    req: HttpRequest, id: web::Path<i64>
) -> impl Responder {
    ProfileImageHandler::get_image(req, id.into_inner())
        .await
}

#[derive(MultipartForm)]
pub struct ProfileImageUpload {
    #[multipart(limit = "3MB")]
    pub image: TempFile
}

/// Set profile image of some user
#[post("/profile-image")]
pub async fn set_profile_image(
    user_id: UserIdReq,
    MultipartForm(form): MultipartForm<ProfileImageUpload>,
) -> impl Responder {
    ProfileImageHandler::set_image(user_id.0, form)
        .await
}

/// Remove profile image of user requesting
#[post("/delete-profile-image")]
pub async fn delete_profile_image(
    user_id: UserIdReq,
) -> impl Responder {
    ProfileImageHandler::remove_image(user_id.0)
        .await
}
