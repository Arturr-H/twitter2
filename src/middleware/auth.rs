/* Imports */
use std::time::{Duration, SystemTime};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};

use crate::error::Error;

/* Constants */
lazy_static::lazy_static! {
    static ref JWT_TOKEN_MAXAGE: u64 = env!("JWT_TOKEN_MAXAGE").parse::<u64>().unwrap();
    static ref JWT_TOKEN_KEY: &'static str = env!("JWT_TOKEN_KEY");
}

/// We call them claims because it's up to the `is_valid`
/// function to see if the claims are claims or facts.
#[derive(Deserialize, Serialize)]
pub struct UserClaims {
    pub handle: String,
    pub id: i64,
    pub exp: usize
}

impl UserClaims {
    /// `id` needs to be the `SERIAL PRIMARY KEY` retrieved
    /// from psotgresql
    pub fn new(handle: String, id: i64) -> Self {
        // `JWT_TOKEN_MAXAGE` days
        let next = SystemTime::now() + Duration::from_secs(*JWT_TOKEN_MAXAGE as u64 * 24 * 60 * 60);
        let exp = next.duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::default()).as_secs() as usize;
        Self { handle, exp, id }
    }

    /// Returns claims if valid
    pub fn is_valid(token: &str) -> Result<TokenData<UserClaims>, Error> {
        jsonwebtoken::decode::<Self>(
            &token,
            &DecodingKey::from_secret((&*JWT_TOKEN_KEY).as_bytes()),
            &Validation::new(jsonwebtoken::Algorithm::HS256)
        ).map_err(Error::new)
    }

    /// Encodes the claims into the JWT token string
    pub fn to_string(&self) -> Option<String> {
        encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
            &self,
            &EncodingKey::from_secret((&*JWT_TOKEN_KEY).as_bytes())
        ).ok()
    }
}
