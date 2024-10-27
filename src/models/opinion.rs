/* Imports */
use serde::Serialize;
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;
use crate::error::Error;

/* Constants */
const OPINION_MAX_LEN: usize = 12;

/// Opinions are snippets of text
/// e.g "amazing", all lowercase which has
/// votes. The N highest voted opinions will
/// be shown below some posts, to highlight the
/// general opinion of a post (one opinion vote
/// per person per post)
#[derive(Serialize)]
pub struct Opinion {
    opinion: String,
    id: i64,
    voted: bool
}

impl Opinion {
    pub fn parse(from: &str) -> Option<String> {
        let new_string = from.to_lowercase();
        let count = UnicodeSegmentation::graphemes(new_string.as_str(), true).count();
        if from.is_empty() { return None };
        if count > OPINION_MAX_LEN { return None };

        Some(new_string)
    }

    /// Vote for an opinion (max 1 per post per user)
    pub async fn set_vote(
        pool: &PgPool, post_id: i64, opinion_id: i64,
        user_id: i64, wants_vote: bool
    ) -> Result<(), Error> {
        let has_voted = sqlx::query!(r"
            SELECT * FROM post_opinion_votes WHERE
            post_opinion_votes.user_id = $1 AND
            post_opinion_votes.opinion_id = $2",
            user_id, opinion_id
        )
            .fetch_optional(pool)
            .await
            .map_err(Error::new)?
            .is_some();
    
        // If we should increment the vote count or decrease it
        let increment: bool;
    
        // If we want to vote but we've already done it or opposite
        // (only for making code clear it's not neccesary)
        if wants_vote && has_voted || !wants_vote && !has_voted {
            return Ok(());
        }else if wants_vote && !has_voted {
            sqlx::query!(r#"
                INSERT INTO post_opinion_votes
                (user_id, post_id, opinion_id) VALUES ($1, $2, $3)"#,
                user_id, post_id, opinion_id
            )
                .execute(pool)
                .await
                .map_err(Error::new)?;
    
            increment = true;
        }else if !wants_vote && has_voted {
            sqlx::query!(r#"
                DELETE FROM post_opinion_votes
                    WHERE post_opinion_votes.user_id = $1
                    AND post_opinion_votes.opinion_id = $2"#,
                user_id, opinion_id
            )
                .execute(pool)
                .await
                .map_err(Error::new)?;
    
            increment = false;
        }else {
            unreachable!()
        }
    
        sqlx::query(&format!(r#"
            UPDATE post_opinions
                SET votes = votes {}
                WHERE post_opinions.id = {}"#,
            if increment { "+ 1" } else { "- 1" },
            opinion_id
        ))
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(Error::new)
    }

    /// Get 5 highest voted opinions for a post
    pub async fn get_post_opinions(pool: &PgPool, post_id: i64) -> Result<Vec<Self>, Error> {
        sqlx::query_as!(Opinion, r#"
            SELECT
                post_opinions.id, post_opinions.opinion,
                is_not_null(post_opinion_votes.user_id) AS "voted!: bool"
            FROM
                post_opinions
            LEFT JOIN post_opinion_votes
                ON post_opinion_votes.post_id = post_opinions.post_id
                AND post_opinion_votes.user_id = post_opinions.user_id
            WHERE
                post_opinions.post_id = $1
            ORDER BY post_opinions.votes DESC
                LIMIT 5;
        "#, post_id)
        .fetch_all(pool).await
        .map_err(Error::new)
    }
}
