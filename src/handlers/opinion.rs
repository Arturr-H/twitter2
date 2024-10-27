//! This file handles endpoints managing post 
//! opinions. Opinions are snippets of text
//! e.g "amazing", all lowercase which has
//! votes. The N highest voted opinions will
//! be shown below some posts, to highlight the
//! general opinion of a post (one opinion vote
//! per person per post)

/* Imports */
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::{error::Error, models::{opinion::Opinion, user::{User, UserIdReq}}, AppData};

/* Structs */
#[derive(Deserialize)]
struct CreateOpinionRequest {
    post_id: i64,
    opinion: String,
}

#[derive(Deserialize)]
struct SetVoteRequest {
    vote: bool,
    post_id: i64,
    opinion_id: i64
}

#[post("/create")]
pub async fn create(
    data: web::Data<AppData>,
    body: web::Json<CreateOpinionRequest>,
    user_id: UserIdReq
) -> impl Responder {
    let user_id = user_id.0;
    let content = Opinion::parse(&body.opinion)
        .ok_or(Error::new("Invalid opinion content"))?;

    let id = sqlx::query_scalar!(r#"
        INSERT INTO post_opinions
        (opinion, post_id, user_id, votes) VALUES ($1, $2, $3, 1)
        returning id
    "#, content, body.post_id, user_id)
    .fetch_one(&data.db)
    .await
    .map_err(Error::new)?;

    /* The person who created the opinion
        will vote for it automatically */
    sqlx::query!(r#"
        INSERT INTO post_opinion_votes
        (opinion_id, post_id, user_id) VALUES ($1, $2, $3)
    "#, id, body.post_id, user_id)
    .execute(&data.db)
    .await.map_err(Error::new)
    .map(|_| HttpResponse::Ok())
}

#[post("/set-vote")]
pub async fn set_vote(
    data: web::Data<AppData>,
    body: web::Json<SetVoteRequest>,
    user_id: UserIdReq
) -> impl Responder {
    Opinion::set_vote(&data.db, body.post_id, body.opinion_id, user_id.0, body.vote)
        .await.map(|_| HttpResponse::Ok())
}

#[get("/get-opinions/{post_id}")]
pub async fn get_opinions(
    data: web::Data<AppData>,
    post_id: web::Path<i64>,
    user_id: UserIdReq
) -> impl Responder {
    Opinion::get_post_opinions(&data.db, post_id.into_inner())
        .await
        .and_then(|e|
            serde_json::to_string(&e)
                .map_err(Error::new)
        )
}
