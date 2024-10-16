#![allow(dead_code, unused_imports, unused_variables)]

/* Modules */
mod models;
mod middleware;
mod utils;
mod handlers;
mod error;

/* Imports */
use actix_cors::Cors;
use actix_web::{get, http::header, web::{self, Data}, App, HttpServer, Responder};
use std::error::Error;
use models::{post::Post, user::User};
use sqlx::PgPool;
use utils::logger::log;
use handlers::{auth, bookmarks, hashtag, post};

/* Constants */
const DATABASE_URL: &'static str = env!("DATABASE_URL");

pub struct AppData {
    db: PgPool
}

#[tokio::main]
async fn main() -> () {
    log("PgPool", "Initializing");
    let pool = PgPool::connect(&DATABASE_URL).await.unwrap();

    // User::create_account(&pool, "artur.h".into(), "arre".into(), "artur.hoffsummer@icloud.com".into(), "password".into()).await.unwrap();

    log("HttpServer", "Initializing");
    HttpServer::new(move || {
        let cors = Cors::permissive()
            // .allowed_origin("http://0.0.0.0:8080")
            /*.allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials()*/;

        App::new()
            .wrap(cors)
            .app_data(Data::new(AppData { db: pool.clone() }))
            .service(ping)
            .service(web::scope("/auth")
                .service(auth::login)
                .service(auth::sign_up)
            )
            .service(web::scope("/post")
                .service(post::publish)
                .service(post::set_like)
                .service(post::set_bookmark)
                .service(post::feed)
                .service(post::post_by_id)
                .service(bookmarks::bookmarks)
                // .service(post::reference)
            )
            .service(web::scope("/hashtag")
                .service(hashtag::posts_by_hashtag)
                .service(hashtag::trending_hashtags)
            )

    })
    .bind(("127.0.0.1", 8080))
    .unwrap().run().await.unwrap();
}

#[get("/ping")]
async fn ping() -> impl Responder { "pong" }