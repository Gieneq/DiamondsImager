use actix_multipart::form::{
    tempfile::TempFile, MultipartForm
};

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(rename = "file")]
    pub file: TempFile,
}