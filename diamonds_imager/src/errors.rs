use axum::{
    extract::multipart::MultipartError, 
    http::StatusCode, 
    response::{
        IntoResponse, 
        Response
    }
};

use crate::services::{processing::ProcessingError, ImageStorageServiceError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    UploadImage(#[from] UploadImageError),

    #[error(transparent)]
    ImageStorageServiceError(#[from] ImageStorageServiceError),
    
    #[error(transparent)]
    ProcessingError(#[from] ProcessingError),
}

#[derive(Debug, thiserror::Error)]
pub enum UploadImageError {
    #[error("ImageEmpty")]
    ImageEmpty,

    #[error("FilenameEmpty")]
    FilenameEmpty,

    #[error("Image too wide max={max}, actual={actual}")]
    ImageTooWide {
        max: u32,
        actual: u32,
    },

    #[error("Image too high max={max}, actual={actual}")]
    ImageTooHigh {
        max: u32,
        actual: u32,
    },

    #[error("FilenameMissing")]
    FilenameMissing,

    #[error("FilenameExtensionMissing")]
    FilenameExtensionMissing,

    #[error("MultipartError {0}")]
    MultipartFailed(#[from] MultipartError),

    #[error("FileIOFailed, reason='{0}'")]
    FileIOFailed(#[from] tokio::io::Error),

    #[error("ImageError, reason='{0}'")]
    ImageError(#[from] image::ImageError),
}

#[derive(Debug, thiserror::Error)]
pub enum PaletteExtractError {
    #[error("ImageNotExist")]
    ImageNotExist,

    #[error("Timeouted")]
    Timeouted,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::UploadImage(e) => match e {
                UploadImageError::ImageEmpty => StatusCode::BAD_REQUEST,
                UploadImageError::FilenameEmpty => StatusCode::BAD_REQUEST,
                UploadImageError::ImageTooWide { max: _, actual: _ } => StatusCode::BAD_REQUEST,
                UploadImageError::ImageTooHigh { max: _, actual: _ } => StatusCode::BAD_REQUEST,
                UploadImageError::FilenameMissing => StatusCode::BAD_REQUEST,
                UploadImageError::FilenameExtensionMissing => StatusCode::BAD_REQUEST,
                UploadImageError::MultipartFailed(_) => StatusCode::BAD_REQUEST,
                UploadImageError::FileIOFailed(_) => StatusCode::BAD_REQUEST,
                UploadImageError::ImageError(_) => StatusCode::BAD_REQUEST,
            },
            Self::ProcessingError(e) => match e {
                ProcessingError::Busy => StatusCode::PROCESSING,
                ProcessingError::ServiceFailed => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::ImageStorageServiceError(e) => match e {
                ImageStorageServiceError::FilenameStemMissing => StatusCode::BAD_REQUEST,
                ImageStorageServiceError::FilenameExtensionMissing => StatusCode::BAD_REQUEST,
                ImageStorageServiceError::ImageNotFound => StatusCode::NOT_FOUND,
            },
        };

        let body = axum::Json(serde_json::json!({ "error" : self.to_string()}));
        (status_code, body).into_response()
    }
}
