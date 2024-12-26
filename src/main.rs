#![allow(dead_code, unused_imports, unused_variables)]

/* Modules */
mod models;
mod middleware;
mod utils;
mod handlers;
mod error;

/* Imports */
use actix_cors::Cors;
use actix_web::{dev::Service, get, http::{header, KeepAlive}, middleware::Logger, web::{self, Data, PayloadConfig}, App, HttpServer, Responder};
use std::error::Error;
use sqlx::PgPool;
use models::{post::Post, user::User};
use utils::logger::log;
use handlers::{auth, bookmarks, feed, post, hashtag, user, opinion};

/* Constants */
const DATABASE_URL: &'static str = env!("DATABASE_URL");
const MAX_REQUEST_SIZE: usize = 1_048_576 * 3; // 3MB
const FRONTEND_URL: &'static str = env!("FRONTEND_URL");

pub struct AppData {
    db: PgPool
}

#[tokio::main]
async fn main() -> () {
    log::yellow("PgPool", "Initializing");
    let pool = PgPool::connect(&DATABASE_URL).await.unwrap();

    // TODO THIS MIGHT BE GOOD TO DO
    // sqlx::migrate!("./migrations")
    //     .run(&pool)
    //     .await.unwrap();

    if env!("DEBUG_LOG_ACTIX").parse::<bool>().unwrap() {
        env_logger::init_from_env(
            env_logger::Env::default()
                .default_filter_or("info")
        );
    }

    log::blue("HttpServer", "Initializing");
    HttpServer::new(move || {
        // TODO: Better CORS implemntation than this...
        let cors = Cors::permissive();

        App::new()
            .wrap(Cors::permissive()
                .allow_any_origin()
            )
            .wrap(Logger::default())
            .app_data(Data::new(AppData { db: pool.clone() }))
            .app_data(PayloadConfig::new(MAX_REQUEST_SIZE))
            .service(ping)
            .service(web::scope("/auth")
                .service(auth::login)
                .service(auth::sign_up)
            )
            .service(web::scope("/user")
                .service(user::get_by_id)
                .service(user::get_by_handle)
                .service(user::get_profile_image)
                .service(user::set_profile_image)
                .service(user::delete_profile_image)
                .service(user::set_following)
                .service(user::posts)
                .service(user::profile)
                .service(user::all_handles)
                .service(user::popular)
            )
            .service(web::scope("/post")
                .service(post::publish)
                .service(post::delete)
                .service(post::set_like)
                .service(post::set_bookmark)
                .service(post::post_by_id)
                .service(bookmarks::bookmarks)

                .service(web::scope("/opinion")
                    .service(opinion::create)
                    .service(opinion::set_vote)
                    .service(opinion::get_opinions)
                )
            )
            .service(web::scope("/feed")
                .service(feed::newest)
                .service(feed::for_you)
                .service(feed::popular)
                .service(feed::replies)
                .service(feed::search)
                .service(web::scope("/hashtag")
                    .service(hashtag::posts_by_hashtag)
                    .service(hashtag::trending_hashtags)
                )
            )
    })
    .keep_alive(KeepAlive::Disabled)
    .bind(("0.0.0.0", 8081))
    .unwrap().run().await.unwrap();
}

#[get("/ping")]
async fn ping() -> impl Responder { "pong" }
