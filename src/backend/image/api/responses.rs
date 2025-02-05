use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;

use super::super::services::errors::InputFileError;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

impl ResponseError for InputFileError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.to_string()
        })
    }
}

impl From<InputFileError> for Box<dyn ResponseError> {
    fn from(value: InputFileError) -> Self {
        Box::new(value)
    }
}

#[derive(Debug, Serialize)]
pub struct ImageUploadResult {
    pub new_filename: String,
}

// impl ResponseError for ProcessingFileError {
//     fn status_code(&self) -> actix_web::http::StatusCode {
//         StatusCode::BAD_REQUEST
//     }

//     fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
//         HttpResponse::build(self.status_code()).json(ErrorResponse {
//             error: self.to_string()
//         })
//     }
// }

// impl From<ProcessingFileError> for Box<dyn ResponseError> {
//     fn from(value: ProcessingFileError) -> Self {
//         Box::new(value)
//     }
// }