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
    dmc::PaletteDmc, 
    ImageId
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadImageResult {
    pub id: ImageId,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPaletteResult {
    pub palette: PaletteDmc
}

impl IntoResponse for UploadImageResult {
    fn into_response(self) -> Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}

impl IntoResponse for GetPaletteResult {
    fn into_response(self) -> Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}