pub mod dmc;
pub mod processing;

use std::{
    collections::HashMap, 
    path::Path, sync::Arc
};

use serde::{
    Deserialize, 
    Serialize
};

pub type ImageId = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageStorageMeta {
    pub filename: String,
    pub upload_time: chrono::DateTime<chrono::Utc>,
    pub last_touch_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ImageStorageElement {
    pub image: Arc<image::RgbImage>,
    pub meta: ImageStorageMeta,
}

#[derive(Debug)]
pub struct ImageStorageService {
    images: HashMap<ImageId, ImageStorageElement>,
}

#[derive(Debug, thiserror::Error)]
pub enum ImageStorageServiceError {
    #[error("FilenameStemMissing")]
    FilenameStemMissing,
    
    #[error("FilenameExtensionMissing")]
    FilenameExtensionMissing,

    #[error("ImageNotFound")]
    ImageNotFound,
}

impl ImageStorageService {
    pub fn new() -> Self {
        Self {
            images: HashMap::new()
        }
    }

    fn generate_id(filename: &str) -> Result<ImageId, ImageStorageServiceError> {
        let uploaded_filename_path = Path::new(&filename);

        let uploaded_filename_stem = uploaded_filename_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or(ImageStorageServiceError::FilenameStemMissing)?;
    
        let extension = uploaded_filename_path.extension()
            .and_then(|oss| oss.to_str())
            .ok_or(ImageStorageServiceError::FilenameExtensionMissing)?;
    
        let new_filename: ImageId = format!("{}_{}.{}", uploaded_filename_stem, uuid::Uuid::new_v4(), extension);
        Ok(new_filename)
    }

    pub fn insert_image(&mut self, filename: String, img: image::DynamicImage) -> Result<ImageId, ImageStorageServiceError> {
        let id = Self::generate_id(&filename)?;

        let time_now = chrono::Utc::now();

        self.images.insert(id.clone(), ImageStorageElement {
            image: Arc::new(img.into()),
            meta: ImageStorageMeta { 
                filename, 
                upload_time: time_now, 
                last_touch_time: time_now 
            }
        });

        Ok(id)
    }

    pub fn access_image(&self, id: &ImageId) -> Result<&ImageStorageElement, ImageStorageServiceError> {
        self.images.get(id).ok_or(ImageStorageServiceError::ImageNotFound)
    }

    pub fn get_image_meta(&self, id: &ImageId) -> Result<ImageStorageMeta, ImageStorageServiceError> {
        self.images.get(id)
            .ok_or(ImageStorageServiceError::ImageNotFound)
            .map(|e| e.meta.clone())
    }

    pub fn remove_image(&mut self, id: &ImageId) -> Result<(), ImageStorageServiceError> {
        if self.images.remove(id).is_none() {
            Err(ImageStorageServiceError::ImageNotFound)
        } else {
            Ok(())
        }
    }
}

