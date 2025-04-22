use axum::{
    extract::multipart::MultipartError, 
    http::StatusCode, 
    response::{
        IntoResponse, 
        Response
    }
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    UploadImage(#[from] UploadImageError),
    
    #[error(transparent)]
    PaletteExtract(#[from] PaletteExtractError),
}

#[derive(Debug, thiserror::Error)]
pub enum UploadImageError {
    #[error("ImageEmpty")]
    ImageEmpty,

    #[error("Image too wide max={max}, actual={actual}")]
    ImageTooWide {
        max: usize,
        actual: usize,
    },

    #[error("Image too high max={max}, actual={actual}")]
    ImageTooHigh {
        max: usize,
        actual: usize,
    },

    #[error("FilenameMissing")]
    FilenameMissing,

    #[error("FilenameExtensionMissing")]
    FilenameExtensionMissing,

    #[error("MultipartError {0}")]
    MultipartFailed(#[from] MultipartError),

    #[error("FileIOFailed, reason='{0}'")]
    FileIOFailed(#[from] tokio::io::Error),
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
            Self::UploadImage(_) => StatusCode::BAD_REQUEST,
            Self::PaletteExtract(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = axum::Json(serde_json::json!({ "error" : self.to_string()}));
        (status_code, body).into_response()
    }
}
