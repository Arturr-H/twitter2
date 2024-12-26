use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;
use crate::{error::Error, models::{tomato::reset_daily_tomatoes, user::UserIdReq}, AppData};

#[derive(serde::Deserialize)]
struct ThrowTomatoRequest {
    post_id: i64,
}

#[post("/tomato")]
pub async fn throw_tomato(
    UserIdReq(user_id): UserIdReq,
    payload: web::Json<ThrowTomatoRequest>,
    data: web::Data<AppData>
) -> impl Responder {
    reset_daily_tomatoes(user_id, &data.db).await.map_err(Error::new)?;

    let tomatoes = sqlx::query_scalar!(
        "SELECT tomatoes FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(Error::new)?;

    if tomatoes <= 0 {
        return Err(Error::new("You have no tomatoes to throw!"));
    }

    let result = sqlx::query!(
        "INSERT INTO post_tomatoes (user_id, post_id) VALUES ($1, $2)",
        user_id,
        payload.post_id
    )
    .execute(&data.db)
    .await
    .map_err(Error::new)?;

    /* Decrement user tomato count */
    sqlx::query!(
        "UPDATE users SET tomatoes = tomatoes - 1 WHERE id = $1",
        user_id
    ).execute(&data.db).await.map_err(Error::new)?;

    /* Increment post tomatoes */
    sqlx::query!(
        "UPDATE posts SET total_tomatoes = total_tomatoes + 1 WHERE id = $1",
        payload.post_id
    ).execute(&data.db).await
    .map_err(Error::new)
    .map(|_| HttpResponse::Ok())
}

#[get("/tomatoes")]
pub async fn get_tomatoes(
    UserIdReq(user_id): UserIdReq,
    data: web::Data<AppData>
) -> Result<impl Responder, Error> {
    struct Tomatoes {
        tomatoes: i32,
        last_tomato_reset: chrono::DateTime<chrono::Utc>,
    }
    let tomatoes = sqlx::query_as!(Tomatoes,
        "SELECT tomatoes, last_tomato_reset FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(Error::new)?;

    Ok(HttpResponse::Ok().json(json!({
        "tomatoes": tomatoes.tomatoes,
        "seconds_until_reset": (tomatoes.last_tomato_reset + chrono::Duration::days(1))
            .signed_duration_since(chrono::Utc::now())
            .num_seconds()
    })))
}
