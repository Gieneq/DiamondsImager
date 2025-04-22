use std::path::Path;
use std::sync::Arc;

use axum::extract;
use axum::{
    response::Html,
    extract::Multipart
};
use serde::{
    Deserialize, 
    Serialize
};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use crate::app::AppData;
use crate::errors::{
    AppError, 
    UploadImageError
};
use crate::results::{
    PaletteResult, 
    UploadImageResult
};
use crate::services::ImageId;


#[derive(Debug, Deserialize, Serialize)]
struct RgbColor(u8, u8, u8);

#[derive(Debug, Serialize, Deserialize)]
pub struct Palette {
    colors: Vec<RgbColor>
}

pub async fn overall_status() -> Html<&'static str> {
    Html("<h1>Diamonds imager is running!</h1>")
}

pub async fn upload_image(
    extract::State(app_data): extract::State<Arc<AppData>>,
    mut multipart: Multipart
) -> Result<UploadImageResult, AppError> {    
    let field = multipart.next_field()
        .await
        .map_err(UploadImageError::from)?;

    let field = field.ok_or(UploadImageError::ImageEmpty)?;
    tracing::trace!("{field:?}");

    let uploaded_filename = field.file_name()
        .ok_or(UploadImageError::FilenameMissing)?
        .to_string();

    let uploaded_filename_path = Path::new(&uploaded_filename);

    let uploaded_filename_stem = uploaded_filename_path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or(UploadImageError::FilenameMissing)?;

    let extension = uploaded_filename_path.extension()
        .and_then(|oss| oss.to_str())
        .ok_or(UploadImageError::FilenameExtensionMissing)?;

    let new_filename: ImageId = format!("{}_{}.{}", uploaded_filename_stem, Uuid::new_v4(), extension);
    let new_filepath = app_data.uplad_dir.join(&new_filename);

    let field_bytes = field.bytes()
        .await
        .map_err(UploadImageError::from)?;

    let mut file = tokio::fs::File::create_new(&new_filepath)
        .await
        .map_err(UploadImageError::from)?;

    file.write_all(&field_bytes)
        .await
        .map_err(UploadImageError::from)?;

    file.flush()
        .await
        .map_err(UploadImageError::from)?;

    let (width, height) = {
        let image = image::open(&new_filepath).unwrap();
        (image.width() as usize, image.height() as usize)
    };

    // Check image size
    let image_size_err = if width > app_data.image_max_width {
        Some(UploadImageError::ImageTooWide { max: app_data.image_max_width, actual: width })
    } else if height > app_data.image_max_height {
        Some(UploadImageError::ImageTooHigh { max: app_data.image_max_width, actual: width })
    } else {
        None
    };

    if let Some(e) = image_size_err {
        tokio::fs::remove_file(&new_filepath)
        .await
        .ok(); // Dont really care
        return Err(e.into());
    };

    tracing::info!("uploading new_filepath={new_filepath:?}, size=({width}, {height})");

    Ok(UploadImageResult::new(
        new_filename,
        width,
        height
    ))
}

pub async fn dmc_full_palette(
    extract::State(app_data): extract::State<Arc<AppData>>
) -> PaletteResult {
    PaletteResult(app_data
        .dmc_full_palette
        .clone()
        .into())
}

pub async fn processings_status(
    extract::State(app_data): extract::State<Arc<AppData>>
) {
    
}