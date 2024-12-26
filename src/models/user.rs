/* Imports */
use std::{future::{ready, Future, Ready}, pin::Pin, sync::Arc};
use actix_files::NamedFile;
use actix_web::{http::{header::ContentType, StatusCode}, web::{self, Data}, FromRequest, HttpRequest, HttpResponse, Responder};
use rand::{thread_rng, Rng};
use regex::Regex;
use serde::Serialize;
use sha2::{Sha256, Digest};
use sqlx::{prelude::FromRow, types::chrono::{self, NaiveDateTime}, PgPool};
use unicode_segmentation::UnicodeSegmentation;
use crate::{error::Error, middleware::auth::UserClaims, utils::logger::log, AppData};
use super::pfp::ProfileImageHandler;

/* Constants */
const EMAIL_REGEX: &'static str = r#"^[\w\.-]+@[\w\.-]+\.\w+$"#;
const HANDLE_REGEX: &'static str = "^[a-z0-9.]+$";
const HANDLE_MAX_LEN: usize = 25;
const HANDLE_MIN_LEN: usize = 3;
const DISPLAYNAME_MAX_LEN: usize = 50;
const DISPLAYNAME_MIN_LEN: usize = 1;
const PASSWORD_MAX_LEN: usize = 45;
const PASSWORD_MIN_LEN: usize = 7;
const PEPPER: &'static str = env!("PEPPER");

#[derive(Debug)]
pub struct User {
    id: i64,

    /// Their @username handle
    handle: String,

    /// Their displayname that can contain e.g
    /// emojies etc.
    displayname: String,

    joined: chrono::DateTime<chrono::Utc>,
    email: String,

    /// SHA-256(pass + salt + pepper)
    /// #[serde(skip)]
    hash: Vec<u8>,
    salt: String,

    followers: i32,
    following: i32,

    tomatoes: i32,
    last_tomato_reset: chrono::DateTime<chrono::Utc>,
}

/// The version of the user struct that does not 
/// spoil any sensitive data
#[derive(Debug, FromRow, sqlx::Type, Serialize)]
pub struct UserInfo {
    pub user_id: i64,
    pub handle: String,
    pub displayname: String,
    pub followers: i32,
    pub following: i32,

    /// If the user requesting is is following the person
    pub is_followed: bool,
}

/// Used for actix web enpoint parameter for only
/// retrieving the user_id of the person sending
/// the request, is way faster than selecting `User`,
/// and more appropriate when you only want to 
/// prevent non authenticated people from calling
/// an endpoint
pub struct UserIdReq(pub i64);

impl User {
    /// Try to create a user. Will fail if:
    /// 
    /// - Email invalid, or in use
    /// - Handle in valid or in use
    /// - Displayname invalid
    /// - Password invalid or too weak
    async fn try_create(
        pool: &PgPool, handle: String, displayname: String,
        email: String, password: String, pepper: &str
    ) -> Result<Self, Error> {
        Self::handle_valid(pool, &handle).await?;
        Self::displayname_valid(&displayname)?;
        Self::email_valid(pool, &email).await?;
        Self::password_valid(&password)?;

        let id = 0;
        let salt = Self::generate_salt();
        let hash = Self::hash_password(&password, &salt);

        Ok(Self {
            // Not yet known, will be determined
            // after inserting
            id,
            joined: chrono::DateTime::from_timestamp_nanos(0),
            followers: 0,
            following: 0,

            handle,
            displayname,
            email,
            hash,
            salt,

            tomatoes: 5,
            last_tomato_reset: chrono::Utc::now()
        })
    }

    /// To non-sensitive data
    pub fn to_non_sensitive(self) -> UserInfo {
        UserInfo {
            user_id: self.id,
            handle: self.handle,
            displayname: self.displayname,
            followers: self.followers,
            following: self.following,

            // This method is only called when the user
            // is the one sending the request, so we can
            // assume that the user is not following
            // themselves
            is_followed: false
        }
    }

    /// Retrieve user from db via id
    pub async fn from_id(pool: &PgPool, id: i64) -> Option<Self> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE users.id = $1", id)
            .fetch_optional(pool)
            .await.ok().flatten()
    }

    /// Create account
    /// Ok yields JWT, Err yields error message
    pub async fn create_account(
        pool: &PgPool, handle: String, displayname: String,
        email: String, password: String, 
    ) -> Result<String, Error> {
        log::bright_green("create_account", format!("Creating account for @{}", &handle));
        let user = Self::try_create(pool, handle.clone(), displayname, email, password, PEPPER).await?;
        log::bright_green("create_account", "Inserting");

        sqlx::query_scalar!(r#"
            INSERT INTO users
            (handle, displayname, email, hash, salt)
            VALUES ($1, $2, $3, $4, $5) RETURNING users.id"#,
            user.handle, user.displayname, user.email, user.hash, user.salt
        )
        .fetch_one(pool)
        .await
        .map_err(|_| Error::new("Could not create account"))

        /* For Result::Ok(_) */
        .and_then(|user_id| {
            UserClaims::new(handle, user_id)
                .to_string().ok_or(Error::new("Could not create JWT token"))
        })
    }

    /// Try login with password
    /// ?: Maybe do something to avoid hackers being able 
    /// ?: differentiate between invalid password and invalid
    /// ?: email - because that can help attackers brute
    /// ?: forcing passwords / getting email addresses
    pub async fn login(pool: &PgPool, email: &String, password: &String) -> Result<String, Error> {
        log::bright_green("login", "Logging in");

        let invalid_pass_or_email = Error::new("Invalid email or password");
        let user = match sqlx::query_as!(Self,
            "SELECT * FROM users WHERE users.email = $1", email
        ).fetch_optional(pool).await {
            Ok(e) => match e {
                Some(row) => row,
                None => return Err(invalid_pass_or_email)
            },
            Err(e) => return Err(Error::new(e))
        };

        log::bright_green("login", "Checking hash");
        let hash = Self::hash_password(password, &user.salt);
        if user.hash == hash {
            log::bright_green("login", "Hash matched - trying to return JWT");
            let claims = UserClaims::new(user.handle, user.id);
            match claims.to_string() {
                Some(e) => Ok(e),
                None => Err(Error::new("Could not generate JWT token"))
            }
        }else {
            Err(invalid_pass_or_email)
        }        
    }

    /// Set following user to true or not
    pub async fn set_following(
        pool: &PgPool, follower_id: i64, followee_id: i64,
        wants_follow: bool
    ) -> Result<(), Error> {
        let is_following = sqlx::query!(r"
            SELECT * FROM follows WHERE
            follows.follower_id = $1 AND
            follows.followee_id = $2",
            follower_id, followee_id
        )
            .fetch_optional(pool)
            .await
            .map_err(Error::new)?
            .is_some();

        // If we should increment the following count or decrease it
        let increment: bool;

        // If we want to like but we've already done it or opposite
        // (only for making code clear it's not neccesary)
        if wants_follow && is_following || !wants_follow && !is_following {
            return Ok(());
        }else if wants_follow && !is_following {
            sqlx::query!(r#"
                INSERT INTO follows
                (follower_id, followee_id) VALUES ($1, $2)"#,
                follower_id, followee_id
            )
                .execute(pool)
                .await
                .map_err(Error::new)?;

            increment = true;
        }else if !wants_follow && is_following {
            sqlx::query!(r#"
                DELETE FROM follows
                    WHERE follows.follower_id = $1
                    AND follows.followee_id = $2"#,
                follower_id, followee_id
            )
                .execute(pool)
                .await
                .map_err(Error::new)?;

            increment = false;
        }else {
            unreachable!()
        }

        let op_str = if increment { "+ 1" } else { "- 1" };

        /* Followers count */
        sqlx::query(&format!(r#"
            UPDATE users
                SET followers = followers {}
                WHERE users.id = {}"#,
            if increment { "+ 1" } else { "- 1" },
            followee_id
        ))
        .execute(pool).await
        .map(|_| ()).map_err(Error::new)?;

        /* Following count */
        sqlx::query(&format!(r#"
            UPDATE users
                SET following = following {}
                WHERE users.id = {}"#,
            if increment { "+ 1" } else { "- 1" },
            follower_id
        ))
        .execute(pool).await
        .map(|_| ()).map_err(Error::new)
    }

    /// Check if JWT is valid and return user if found via appdata postgres pool
    async fn from_appdata(pool: &AppData, jwt: String) -> Result<Self, Error> {
        let user_claims = UserClaims::is_valid(&jwt)?;
        let id = user_claims.claims.id;

        sqlx::query_as!(Self,
            "SELECT * FROM users WHERE users.id = $1", id
        )
        .fetch_optional(&pool.db).await
        .map_err(Error::new)
        .and_then(|e| e.ok_or(Error::new("User not found")))
    }

    /// Length checks and char checks for handle (username)
    async fn handle_valid(pool: &PgPool, handle: &String) -> Result<(), Error> {
        // We won't need to do UnicodeSegmentation::graphemes here
        // like we do in the `displayname_valid` method because
        // the regex has strict rules on what characters are ok.
        let len = handle.len();
        if len < HANDLE_MIN_LEN {
            return Err(Error::new("Handle must be at least 3 characters long"))
        }if len > HANDLE_MAX_LEN {
            return Err(Error::new("Handle must be less than 15 characters long"))
        }

        let handle_rgx = Regex::new(HANDLE_REGEX).unwrap();
        if !handle_rgx.is_match(handle) {
            return Err(Error::new("Handle must only contain either characters a-z, 0-9 or a \".\""))
        }

        match Self::handle_occupied(pool, handle).await? {
            // Not occupied
            false => Ok(()),

            // Occupied
            true => Err(Error::new("Handle occupied"))
        }
    }

    /// Checks if handle is in use 
    async fn handle_occupied(pool: &PgPool, handle: &String) -> Result<bool, Error> {
        let result = sqlx::query!("SELECT FROM users WHERE handle = $1", handle)
            .fetch_optional(pool).await;

        match result {
            Ok(e) => Ok(e.is_some()),
            Err(_) => Err(Error::new("Could not check for occupation"))
        }
    }
    /// Checks if email is in use 
    async fn email_occupied(pool: &PgPool, email: &String) -> Result<bool, Error> {
        let result = sqlx::query!("SELECT FROM users WHERE email = $1", email)
            .fetch_optional(pool).await;

        match result {
            Ok(e) => Ok(e.is_some()),
            Err(_) => Err(Error::new("Could not check for occupation"))
        }
    }

    /// Displayname length checks
    fn displayname_valid(displayname: &String) -> Result<(), Error> {
        let len = UnicodeSegmentation::graphemes(displayname.as_str(), true).count();
        if len < DISPLAYNAME_MIN_LEN {
            return Err(Error::new(format!("Displayname must be at least {} characters long", DISPLAYNAME_MIN_LEN)))
        }if len > DISPLAYNAME_MAX_LEN {
            return Err(Error::new(format!("Displayname must be less than {} characters long", DISPLAYNAME_MAX_LEN)))
        }

        Ok(())
    }
    /// Password length checks
    fn password_valid(password: &String) -> Result<(), Error> {
        let len = UnicodeSegmentation::graphemes(password.as_str(), true).count();
        if len < PASSWORD_MIN_LEN {
            return Err(Error::new(format!("Password must be at least {} characters long", PASSWORD_MIN_LEN)))
        }if len > PASSWORD_MAX_LEN {
            return Err(Error::new(format!("Password must be less than {} characters long", PASSWORD_MAX_LEN)))
        }

        Ok(())
    }

    /// Email regex checks
    async fn email_valid(pool: &PgPool, email: &String) -> Result<(), Error> {
        let regex = Regex::new(EMAIL_REGEX).unwrap();
        match regex.is_match(&email) {
            true => {
                match Self::email_occupied(pool, email).await? {
                    true => Err(Error::new("Email already in use")),
                    false => Ok(())
                }
            },
            false => Err(Error::new("Email is invalid"))
        }
    }

    /// Hashes `password` with salt and pepper
    fn hash_password(password: &String, salt: &String) -> Vec<u8> {
        let inner = password.to_owned() + salt + PEPPER;
        let mut hasher = Sha256::new();
        hasher.update(inner);
        hasher.finalize()[..].to_owned()
    }

    /// Generates a salt which is stored in the user table
    fn generate_salt() -> String {
        let chars = "abcdefghijklmnopqrstuvwxyz1234567890+-.,;:!\"#€%&/()=?*".chars().collect::<Vec<char>>();
        let mut rng = thread_rng();
        let length = rng.gen_range(25..=35);
        let mut end = String::with_capacity(length);
        for _ in 0..length {
            end.push(chars[rng.gen_range(0..chars.len())])
        }
        end
    }

    // Getters
    pub fn id(&self) -> i64 { self.id }
    pub fn displayname(&self) -> &String { &self.displayname }
    pub fn handle(&self) -> &String { &self.handle }
}

impl FromRequest for User {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization").cloned();
        let appdata = match req.app_data::<web::Data<AppData>>() {
            Some(e) => e.clone(),
            None => return Box::pin(async { Err(Error::new_with_code(
                "Internal server error (appdata retrieval from request)",
                StatusCode::INTERNAL_SERVER_ERROR
            ))})
        };

        Box::pin(async move {
            if let Some(header_value) = auth_header {
                if let Ok(header_str) = header_value.to_str() {
                    if let Some(token) = header_str.strip_prefix("Bearer ") {
                        return User::from_appdata(appdata.get_ref(), token.to_string()).await
                    }
                }
            }
            
            Err(Error::new_with_code("Unauthorized", StatusCode::UNAUTHORIZED))
        })
    }
}

impl FromRequest for UserIdReq {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization").cloned();
        if let Some(header_value) = auth_header {
            if let Ok(header_str) = header_value.to_str() {
                if let Some(token) = header_str.strip_prefix("Bearer ") {
                    return match UserClaims::is_valid(&token) {
                        Ok(e) => ready(Ok(Self(e.claims.id))),
                        Err(e) => ready(Err(e))
                    }
                }
            }
        }
        
        ready(Err(Error::new_with_code("Unauthorized", StatusCode::UNAUTHORIZED)))
    }
}
