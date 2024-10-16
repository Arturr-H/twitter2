/* Imports */
use actix_web::{error::{ErrorInternalServerError, HttpError}, get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use crate::{error::Error, models::user::User, AppData};

/* Structs */
#[derive(Deserialize)]
struct LoginRequest { email: String, password: String }
#[derive(Deserialize)]
struct SignUpRequest {
    email: String, password: String, handle: String,
    displayname: String
}

/// Responds with the JWT of the user
#[post("/login")]
pub async fn login(
    req: HttpRequest, data: web::Data<AppData>,
    body: web::Json<LoginRequest>
) -> impl Responder {
    let jwt = User::login(&data.db, &body.email, &body.password)
        .await?;

    Ok::<_, Error>(
        HttpResponse::Ok()
            .json(json!({ "token": jwt }))
    )
}

/// Responds with the JWT of the user
#[post("/sign-up")]
pub async fn sign_up(
    req: HttpRequest, data: web::Data<AppData>,
    body: web::Json<SignUpRequest>
) -> impl Responder {
    let jwt = User::create_account(
        &data.db, body.handle.clone(), body.displayname.clone(),
        body.email.clone(), body.password.clone()).await?;

    Ok::<_, Error>(
        HttpResponse::Ok()
            .json(json!({ "token": jwt }))
    )
}

