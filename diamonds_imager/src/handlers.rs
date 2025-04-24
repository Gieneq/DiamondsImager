use std::sync::Arc;

use axum::extract;

use axum::{
    response::Html,
    extract::Multipart
};

use crate::app::AppData;

use crate::errors::{
    AppError, 
    UploadImageError
};
use crate::results::UploadImageResult;

pub async fn overall_status() -> Html<&'static str> {
    Html("<h1>Diamonds imager is running!</h1>")
}

pub async fn upload_image(
    extract::State(app_data): extract::State<Arc<AppData>>,
    mut multipart: Multipart
) -> Result<UploadImageResult, AppError> {    
    let Some(field) = multipart.next_field().await.map_err(UploadImageError::from)? else {
        return Err(UploadImageError::ImageEmpty.into());
    };

    // Extract image and meta from uploaded field
    let uploaded_filename = field.file_name()
    .ok_or(UploadImageError::FilenameMissing)?
    .to_string();

    if uploaded_filename.is_empty() {
        return Err(UploadImageError::FilenameEmpty.into());
    }

    let bytes = field.bytes().await.map_err(UploadImageError::from)?;
    
    let image = image::load_from_memory(&bytes).map_err(UploadImageError::ImageError)?;

    // Check image
    let (width, height) = (image.width(), image.height());

    if width > app_data.image_max_width {
        return Err(UploadImageError::ImageTooWide { max: app_data.image_max_width, actual: width }.into());
    }
    
    if height > app_data.image_max_height {
        return Err(UploadImageError::ImageTooHigh { max: app_data.image_max_height, actual: height }.into());
    }

    let mut image_storage_service_guard = app_data.image_storage_service.lock().await;
    let id = image_storage_service_guard.insert_image(uploaded_filename, image)?;

    Ok(UploadImageResult { id, width, height })
}