/* Imports */
use regex::Regex;
use serde::Serialize;
use sqlx::{prelude::FromRow, types::chrono::NaiveDateTime, PgPool};
use crate::error::Error;
use super::{post_citation::PostCitation, user::{User, UserInfo}};
use chrono::serde::ts_milliseconds_option;

/* Post boolean for keeping track of liked, bookmarked or not */
pub enum PostBoolean { Like, Bookmark }

#[derive(FromRow, Debug, Default, sqlx::Type)]
pub struct Post {
    /// Primary key
    pub id: i64,

    /// The text content of this post. Can
    /// contain links etc.
    pub content: String,

    // These will default to 0
    pub total_likes: i64,
    pub total_replies: i64,
    pub poster_id: i64,

    pub replies_to: Option<i64>,
    pub citation: Option<serde_json::Value>,

    pub created_at: chrono::DateTime<chrono::Utc>
}
#[derive(Serialize, FromRow, sqlx::Type)]
pub struct PostWithUser {
    /* Post info */
    pub id: Option<i64>,
    pub content: Option<String>,
    pub total_likes: Option<i64>,
    pub total_replies: Option<i64>,
    pub poster_id: Option<i64>,
    pub replies_to: Option<Option<i64>>,
    pub citation: Option<Option<serde_json::Value>>,

    #[serde(with = "ts_milliseconds_option")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,

    /* User info */
    pub user_id: Option<i64>,
    pub displayname: Option<String>,
    pub handle: Option<String>,
    
    /* Post metadata related to user */
    pub liked: Option<bool>,
    pub bookmarked: Option<bool>,
    pub is_followed: Option<bool>,

    pub top_opinions: Option<serde_json::Value>,
}

impl Post {
    /// Used before inserting, with id set temporarily
    /// to zero.
    pub fn new(poster_id: i64, content: String, replies_to: Option<i64>, citation: Option<PostCitation>) -> Self {
        let citation = citation.and_then(|e| serde_json::to_value(e).ok());
        Post {
            content,
            poster_id,
            replies_to,
            citation,
            ..Default::default()
        }
    }

    /// Inserts into db, also inserts hashtags.
    pub async fn insert_into(&self, pool: &PgPool) -> Result<(), Error> {
        let (hashtags, _) = self.hashtags_and_mentions();

        // Insert post
        let post_id: i64 = sqlx::query_scalar!(r#"
            INSERT INTO posts
            (content, poster_id, replies_to, citation) VALUES ($1, $2, $3, $4)
            returning id"#,
            self.content, self.poster_id, self.replies_to, self.citation
        ).fetch_one(pool)
        .await
        .map_err(Error::new)?;

        // Insert hashtags
        for tag in hashtags {
            // Try to insert the hashtag, or get its ID if it exists
            let hashtag_id: i64 = sqlx::query_scalar!(r#"
                INSERT INTO hashtags (tag)
                VALUES ($1)
                ON CONFLICT (tag) DO UPDATE SET tag = excluded.tag
                RETURNING id"#,
                tag.to_lowercase()
            )
            .fetch_one(pool)
            .await
            .map_err(Error::new)?;

            // Insert into post_hashtags ("connect" the hashtag with
            // the corresponding post)
            sqlx::query!(r#"
                INSERT INTO post_hashtags (post_id, hashtag_id)
                VALUES ($1, $2)
                ON CONFLICT (post_id, hashtag_id) DO NOTHING"#,
                post_id,
                hashtag_id
            )
            .execute(pool)
            .await
            .map_err(Error::new)?;
        }

        // Increase the total_replies of the post we replied to
        if let Some(replies_to) = self.replies_to {
            sqlx::query!(r#"
                UPDATE posts
                SET total_replies = total_replies + 1
                WHERE id = $1"#,
                replies_to
            )
            .execute(pool)
            .await
            .map_err(Error::new)?;
        }

        Ok(())
    }

    /// The toggler_id is the person who likes / unlikes and the post_id is the
    /// post that will recieve a like if not already existing, same for bookmarks
    pub async fn set_boolean(b: PostBoolean, pool: &PgPool, to: bool, toggler_id: i64, post_id: i32) -> Result<(), Error> {
        let table_name = match b {
            PostBoolean::Bookmark => "post_bookmarks",
            PostBoolean::Like => "post_likes",
        };

        let is_enabled = sqlx::query(&format!(
            r"SELECT * FROM {table_name} WHERE
                {table_name}.post_id = {post_id} AND
                {table_name}.user_id = {toggler_id}",
            ))
            .fetch_optional(pool)
            .await
            .map_err(Error::new)?
            .is_some();

        // If we should increment the boolean on the post.total_likes (only for like)
        let increment: bool;

        // If we want to like but we've already done it or opposite
        // (only for making code clear it's not neccesary)
        if to && is_enabled || !to && !is_enabled {
            return Ok(());
        }else if to && !is_enabled {
            sqlx::query(&format!(r"
                INSERT INTO {table_name} (user_id, post_id) VALUES ({toggler_id}, {post_id})
            "))
                .execute(pool)
                .await
                .map_err(Error::new)?;

            increment = true;
        }else if !to && is_enabled {
            sqlx::query(&format!(r"
                DELETE FROM {table_name}
                    WHERE {table_name}.user_id = {toggler_id}
                    AND {table_name}.post_id = {post_id}"
                )
            )
                .execute(pool)
                .await
                .map_err(Error::new)?;

            increment = false;
        }else {
            unreachable!()
        }

        /* Only liked should increment */
        if matches!(b, PostBoolean::Like) {
            sqlx::query(&format!(r"
                UPDATE posts
                    SET total_likes = total_likes {}
                    WHERE id = {}",
                if increment { "+ 1" } else { "- 1" },
                post_id
            ))
            .execute(pool)
            .await
            .map(|_| ())
            .map_err(Error::new)
        }else {
            Ok(())
        }
    }

    /// Looks through a string and returns (hashtags, mentions)
    pub fn hashtags_and_mentions(&self) -> (Vec<String>, Vec<String>) {
        const HASHTAG_REGEX: &'static str = "^[a-zA-Z0-9]+$";
        let mut hashtags = Vec::new();
        let mut mentions = Vec::new();
        let rgx = Regex::new(HASHTAG_REGEX).unwrap();

        // ish = is seeking hashtag, ism = is seeking mention
        let mut ish = false;
        let mut ism = false;
        let mut curr_char_buf = String::new();

        let set_seeking = &mut |cbuf: &mut String, ish: &mut bool, ism: &mut bool| {
            if !cbuf.is_empty() {
                if *ish { hashtags.push(cbuf.to_lowercase()) }
                else if *ism { mentions.push(cbuf.to_lowercase()) };
            }

            *ish = false;
            *ism = false;
            cbuf.clear();
        };

        for char in self.content.chars() {
            if char == '#' && !ism && !ish {
                ish = true;
            }else if char == '@' && !ism && !ish {
                ism = true;
            }
            
            // If the user begins a new hashtag or mention inside of a
            // hashtag or mention (without whitespace between) like this: 
            // @user1@user2 or #ht1#ht2 or @user1#ht1 or #ht1@user2
            // we need to also treat them as separate
            else if char == '#' && (ism || ish) {
                set_seeking(&mut curr_char_buf, &mut ish, &mut ism);
                ish = true;
                ism = false;
            }else if char == '@' && (ism || ish) {
                set_seeking(&mut curr_char_buf, &mut ish, &mut ism);
                ish = false;
                ism = true;
            }
            
            else if char.is_whitespace() || char.is_ascii_whitespace() {
                set_seeking(&mut curr_char_buf, &mut ish, &mut ism);
            }else if rgx.is_match(&char.to_string()) {
                curr_char_buf.push(char.to_ascii_lowercase());
            }else {
                set_seeking(&mut curr_char_buf, &mut ish, &mut ism);
            }
        }

        if ish || ism { set_seeking(&mut curr_char_buf, &mut ish, &mut ism) }

        (hashtags, mentions)
    }
}