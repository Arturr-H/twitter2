use actix_web::Responder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::FromRow;


/// How many characters we'll include from
/// left to right of the referenced slice
const REFERENCE_PADDING: i32 = 15;

/// Returns a reference to a section of a post.
/// 
/// Imagine this post content:
/// "Hello my name is Artur and I like cookies".
/// Referencing index 27-40 will highlight the
/// string "I like cookies" and show a more 
/// detailed "view" with 15 characters around
/// it which will return:
/// ..."e is Artur and **I like cookies**"
#[derive(Serialize, sqlx::Type, Debug, Default, Deserialize)]
pub struct PostCitation {
    /// The window of (PADDING citation PADDING)
    content_slice: String,

    /// Highlight begin index in content_slice window
    beginning: i32,
    /// Highlight end index in content_slice window
    end: i32,

    // If we need to add ellipsis at the end or beginning
    // outside of padding because it's not containing the
    // full text
    ellipsis_left: bool,
    ellipsis_right: bool,

    post_id: i32,
    user_id: i32,
    displayname: String,
    handle: String,
}

impl PostCitation {
    // pub fn new(content: String, start_ref: i32, end_ref: i32) -> Self {
    //     let start_slice_index = (start_ref - REFERENCE_PADDING).max(0) as i32;
    //     let end_slice_index = (end_ref + REFERENCE_PADDING).min(content.len() as i32) as i32;
    //     let beginning = REFERENCE_PADDING - (REFERENCE_PADDING - start_ref).max(0) as i32;
    //     let end = beginning + end_ref - start_ref as i32;
    //     let content_slice = &content[(start_slice_index as usize)..(end_slice_index as usize)];
    //     let mut ellipsis_left = false;
    //     let mut ellipsis_right = false;

    //     if start_ref >= REFERENCE_PADDING + 1{
    //         ellipsis_left = true;
    //     }if end_ref + REFERENCE_PADDING < content.len() as i32 {
    //         ellipsis_right = true;
    //     }

    //     Self {
    //         content_slice: content_slice.to_string(),
    //         beginning,
    //         end,
    //         ellipsis_right,
    //         ellipsis_left
    //     }
    // }

    pub fn to_json(&self) -> Option<Value> {
        serde_json::to_value(self).ok()
    }
}
