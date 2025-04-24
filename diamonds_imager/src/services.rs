use std::{collections::HashMap, path::Path};

pub type ImageId = String;

// pub struct ProcessingStatus {

// }

// pub struct ProcessingsIndex {
//     pub index: HashMap<ImageId, ProcessingStatus>,
// }

// fn access_image_by_id(id: &ImageId) -> () {
    
// }

// pub fn is_image_id_present(id: &ImageId) -> bool {

// }

#[derive(Debug)]
pub struct ImageStorageElement {
    image: image::RgbImage,
    filename: String,
    // processing status
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

        self.images.insert(id.clone(), ImageStorageElement {
            image: img.into(),
            filename
        });

        Ok(id)
    }
}