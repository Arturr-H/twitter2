use ::chrono::{Duration, TimeZone, Timelike};
use chrono::Utc;
use sqlx::{types::chrono, PgPool};

pub async fn reset_daily_tomatoes(user_id: i64, db: &PgPool) -> sqlx::Result<()> {
    let reset_interval = Duration::days(1);
    let now = Utc::now();

    let user = sqlx::query!(
        "SELECT last_tomato_reset FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(db)
    .await?;

    let last_reset = user.last_tomato_reset;
    
    if now.signed_duration_since(last_reset) > reset_interval {
        sqlx::query!(
            "UPDATE users SET tomatoes = 5, last_tomato_reset = $1 WHERE id = $2",
            now,
            user_id
        )
        .execute(db)
        .await?;
    }

    Ok(())
}
