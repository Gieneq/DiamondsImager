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
use crate::requests::ExtractQueryMaxColorsCount;
use crate::results::{
    FinishPaletteExtractionResult, 
    GetPaletteResult, 
    StartPaletteExtractionResult, 
    UploadImageResult
};

use crate::services::processing::worker::Work;
use crate::services::{
    ImageId, 
    ImageStorageMeta
};

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

pub async fn get_image_meta(
    extract::State(app_data): extract::State<Arc<AppData>>,
    extract::Path(id): extract::Path<ImageId>
) -> Result<ImageStorageMeta, AppError> {
    let image_storage_service_guard = app_data.image_storage_service.lock().await;
    image_storage_service_guard.get_image_meta(&id).map_err(AppError::from)
}

pub async fn delete_image(
    extract::State(app_data): extract::State<Arc<AppData>>,
    extract::Path(id): extract::Path<ImageId>
) -> Result<(), AppError> {
    let mut image_storage_service_guard = app_data.image_storage_service.lock().await;
    image_storage_service_guard.remove_image(&id).map_err(AppError::from)
}

pub async fn get_full_dmc_palette(
    extract::State(app_data): extract::State<Arc<AppData>>
) -> GetPaletteResult { 
    GetPaletteResult {
        palette: app_data.palette_dmc_full.as_ref().clone()
    }
}

pub async fn start_extracting_dmc_palette(
    extract::State(app_data): extract::State<Arc<AppData>>,
    extract::Path(id): extract::Path<ImageId>,
    extract::Query(query_max_colors): extract::Query<ExtractQueryMaxColorsCount>,
) -> Result<(), AppError> {
    let cloned_image = {
        let image_storage_service_guard = app_data.image_storage_service.lock().await;
        let element = image_storage_service_guard.access_image(&id)?;
        element.image.clone()
    };

    let processing_runner_service_guard = app_data.processing_runner_service.lock().await;
    let start_result = processing_runner_service_guard.enque_work(Work::PaletteExtract {
        palette_dmc: app_data.palette_dmc_full.clone(),
        src_image: cloned_image, 
        max_colors: query_max_colors.max_colors
    }).await;

    let work_id = start_result.map_err(AppError::from);
    Ok(())
}

pub async fn poll_finish_extracting_dmc_palette(
    extract::State(app_data): extract::State<Arc<AppData>>,
    extract::Path(id): extract::Path<ImageId>
) -> Result<FinishPaletteExtractionResult, AppError> {
    // let mut image_storage_service_guard = app_data.image_storage_service.lock().await;
    // // TODO consider offloading to blocking, can be CPU intensive
    // let palette = image_storage_service_guard.get_palette_from_image(&id)?;
    // Ok(GetPaletteResult { palette })
    todo!()
}