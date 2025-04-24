use diamonds_gen::dmc::PaletteDmcData;

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

use crate::services::ImageId;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadImageResult {
    pub id: ImageId,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaletteResult(pub PaletteDmcData);

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingStatus {

}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcesingsResult(pub Vec<ProcessingStatus>);

impl IntoResponse for UploadImageResult {
    fn into_response(self) -> Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}

impl IntoResponse for PaletteResult {
    fn into_response(self) -> Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}