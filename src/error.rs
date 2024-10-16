use std::fmt::Display;

use actix_web::{body::BoxBody, http::StatusCode, HttpResponse, Responder, ResponseError};
use serde_json::json;


/// The error struct we use in this application
#[derive(Debug)]
pub struct Error {
    code: StatusCode,
    description: String
}

impl Error {
    /// Creates a new error with code -1 (unspecified)
    pub fn new(description: impl ToString) -> Self {
        Self { code: StatusCode::NOT_ACCEPTABLE, description: description.to_string() }
    }

    /// Creates a new error with code specified
    pub fn new_with_code(description: impl ToString, code: StatusCode) -> Self {
        Self { code, description: description.to_string() }
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.code)
            .json(json!({
                "status": self.code.as_str(),
                "description": self.description
            }))
    }
    fn status_code(&self) -> StatusCode { self.code }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}
