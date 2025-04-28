use std::collections::HashMap;

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

use crate::services::{
    dmc::{Dmc, PaletteDmc}, 
    ImageId, ImageStorageMeta
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadImageResult {
    pub id: ImageId,
    pub width: u32,
    pub height: u32,
}

impl IntoResponse for UploadImageResult {
    fn into_response(self) -> Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPaletteResult {
    pub palette: PaletteDmc
}

impl IntoResponse for GetPaletteResult {
    fn into_response(self) -> Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}

impl IntoResponse for ImageStorageMeta {
    fn into_response(self) -> Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartPaletteExtractionResult {
    pub was_started: bool
}

impl IntoResponse for StartPaletteExtractionResult {
    fn into_response(self) -> Response {
        let status_code = if self.was_started { StatusCode::OK } else { StatusCode::TOO_MANY_REQUESTS };
        let body = axum::Json(self);
        (status_code, body).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishPaletteExtractionResult {
    pub result: Option<HashMap<Dmc, usize>>
}

impl IntoResponse for FinishPaletteExtractionResult {
    fn into_response(self) -> Response {
        let status_code = if self.result.is_some() { StatusCode::OK } else { StatusCode::PROCESSING };
        let body = axum::Json(self);
        (status_code, body).into_response()
    }
}
