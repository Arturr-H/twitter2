/* Imports */
use actix_web::{get, post, route, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use crate::{error::Error, middleware::auth::UserClaims, models::{post::{Post, PostBoolean, PostWithUser}, post_citation::PostCitation, user::{User, UserIdReq, UserInfo}}, utils::logger::log, AppData};

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

/// Get profile image of the current logged in user
#[get("/profile-image-self")]
pub async fn profile_image_self(req: HttpRequest, user_id: UserIdReq) -> impl Responder {
    User::profile_image(req, user_id.0)
}
