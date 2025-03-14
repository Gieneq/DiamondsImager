use serde::{
    Deserialize, 
    Serialize
};

use axum::{
    http::StatusCode, 
    response::{
        IntoResponse, 
        Response
    }
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadImageResult {
    pub id: String,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaletteExtractResult {
    id: String,
}

impl UploadImageResult {
    pub fn new(id: String, width: usize, height: usize) -> Self {
        Self { id, width, height }
    }
}

impl IntoResponse for UploadImageResult {
    fn into_response(self) -> Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}

impl PaletteExtractResult {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl IntoResponse for PaletteExtractResult {
    fn into_response(self) -> Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}