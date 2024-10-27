//! This file handles user profile picture
//! related things

use actix_files::NamedFile;
use actix_web::{http::header::ContentType, web, HttpRequest, HttpResponse, Responder};
use image::{imageops::resize, EncodableLayout, ImageFormat};
use crate::{error::Error, handlers::user::ProfileImageUpload};

/// Handles profile image related things
pub struct ProfileImageHandler;

impl ProfileImageHandler {
    /// Returns HTTP response (from actix) with either
    /// the found pfp or the default-user.jpg file.
    pub async fn get_image(req: HttpRequest, user_id: i64) -> HttpResponse {
        const DEFAULT_USER: &[u8] = include_bytes!("../../assets/images/default-user.jpg");
        let path = String::from("./assets/images/profile/") + &user_id.to_string() + ".jpg";
        match tokio::fs::read(&path).await {
            Ok(e) => HttpResponse::Ok()
                .content_type(ContentType::jpeg())
                .body(e),
            Err(_) => HttpResponse::Ok()
                .content_type(ContentType::jpeg())
                .body(DEFAULT_USER)
        }
    }

    /// Set the profile image for a user
    pub async fn set_image(user_id: i64, form: ProfileImageUpload) -> Result<HttpResponse, Error> {
        dbg!("Setting image");
        let bytes = tokio::fs::read(form.image.file)
            .await
            .map_err(Error::new)?;
        let img = image::load_from_memory(&bytes)
            .map_err(Error::new)?
            .to_rgb8();
        dbg!("Setting image");

        let image = resize(&img, 200, 200, image::imageops::FilterType::Nearest);
        let path = format!("./assets/images/profile/{}.jpg", user_id);
        dbg!("Setting image");
        
        image.save_with_format(path, ImageFormat::Jpeg)
            .map_err(Error::new)
            .map(|_| HttpResponse::Ok().finish())
    }

    /// Remove profile image of user requesting
    pub async fn remove_image(user_id: i64) -> Result<HttpResponse, Error> {
        let path = format!("./assets/images/profile/{}.jpg", user_id);
        tokio::fs::remove_file(path)
            .await
            .map_err(Error::new)
            .map(|_| HttpResponse::Ok().finish())
    }
}
