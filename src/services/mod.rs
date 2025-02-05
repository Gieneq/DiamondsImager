use std::borrow::Cow;

pub fn get_hello_message() -> String {
    "Hello".to_string()
}

pub mod processing {
    use crate::{services::modify_filepath, settings::{Settings, Size}};
    use actix_multipart::form::tempfile::TempFile;
    use image::{imageops::FilterType, DynamicImage, GenericImage, GenericImageView, Pixel};
    
    #[derive(Debug, thiserror::Error)]
    pub enum ProcessingFileError {
        #[error("File width={actual} is too small, should be > {min}.")]
        WidthTooSmall{
            actual: u32,
            min: u32,
        },
        
        #[error("File height={actual} is too small, should be > {min}.")]
        HeightTooSmall{
            actual: u32,
            min: u32,
        },
        
        #[error("File width={actual} is too big, should be <= {max}.")]
        WidthTooBig{
            actual: u32,
            max: u32,
        },
        
        #[error("File height={actual} is too big, should be <= {max}.")]
        HeightTooBig{
            actual: u32,
            max: u32,
        },
        
        #[error("Some unknown error.")]
        OtherError,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum InputFileError {
        #[error("File is empty")]
        FileEmpty,

        #[error("File's name is missing")]
        FileNoname,

        #[error("File could not been saved")]
        FileNotSaved,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Constrins {
        pub diamons_size: f32,
        pub result_size_mm: Size<f32>
    }

    impl Default for Constrins {
        fn default() -> Self {
            Self {
                diamons_size: 2.5,
                result_size_mm: Size {
                    width: 500.0,
                    height: 300.0 
                }
            }
        }
    }

    impl Constrins {
        fn get_size_in_diamonds(&self) -> Size<u32> {
            fn round_down(value: f32) -> u32 {
                value.floor() as u32
            }

            Size {
                width: round_down(self.result_size_mm.width / self.diamons_size),
                height: round_down(self.result_size_mm.height / self.diamons_size)
            }
        }
    }

    pub fn process_image(
        filepath: String,
        min_size: Size<u32>,
        max_size: Size<u32>,
        constrins: &Constrins
    ) -> Result<String, ProcessingFileError> {
        let image = image::open(filepath.clone()).map_err(|_e| {
            ProcessingFileError::OtherError
        })?;

        log::info!("Validating image {}", filepath);
        let image = validate_src_img_size(min_size, max_size, image)?;

        log::info!("Processing image {}", filepath);
        let processed_image = apply_processing(constrins, image);

        let new_filepath = modify_filepath(&filepath, "_prev");

        log::info!("Processing done! Got new image {}. Saving...", new_filepath);

        //save image
        processed_image.save(new_filepath.clone()).map_err(|_e| {
            ProcessingFileError::OtherError
        })?;

        Ok(new_filepath)
    }

    fn validate_src_img_size(
        min_size: Size<u32>,
        max_size: Size<u32>,
        image: DynamicImage
    ) -> Result<DynamicImage, ProcessingFileError> {
        if image.width() <= min_size.width {
            Err(ProcessingFileError::WidthTooSmall { actual: image.width(), min: min_size.width })
        }
        else if image.width() > max_size.width {
            Err(ProcessingFileError::WidthTooBig { actual: image.width(), max: min_size.width })
        }
        else if image.height() <= min_size.height {
            Err(ProcessingFileError::HeightTooSmall { actual: image.height(), min: min_size.height })
        }
        else if image.height() > max_size.height {
            Err(ProcessingFileError::HeightTooBig { actual: image.height(), max: max_size.height })
        }
        else { Ok(image) }
    }

    fn apply_processing(
        constrins: &Constrins,
        image: DynamicImage
    ) -> DynamicImage {
        log::info!("Constrins={:?}", constrins.get_size_in_diamonds());
        let resized = image.resize(
            constrins.get_size_in_diamonds().width,
            constrins.get_size_in_diamonds().height,
            FilterType::CatmullRom
        );

        let mut result = DynamicImage::new(resized.width(), resized.height(), image::ColorType::Rgb8);

        for (x, y, color) in resized.pixels() {
            let color = color.to_rgb();
            result.put_pixel(x, y, color.to_rgba());
        }

        result
    }
    
    pub fn save_file_with_generated_unique_filename(settings: &Settings, file: TempFile) -> Result<String, InputFileError> {
        let unique_filename = create_unique_filename(file.file_name.as_ref().map(|s| s.as_str()))?;
        let unique_filepath = create_unique_filepath(settings, &unique_filename);
        log::info!("saving to '{unique_filepath}'");
        match file.file.persist(unique_filepath.clone()) {
            Ok(_) => Ok(unique_filename),
            Err(e) => {
                log::error!("Could not save file, reason {e}");
                Err(InputFileError::FileNotSaved)
            }
        }
        
    }

    fn create_unique_filename(filename: Option<&str>) -> Result<String, InputFileError> {
        let file_name_full = filename.ok_or(InputFileError::FileNoname)?;
        let mut file_split = file_name_full.split(".");

        let result = format!("{}_{}.{}", 
            file_split.next().unwrap(),
            uuid::Uuid::new_v4(),
            file_split.next().unwrap(),
        );
        Ok(result)
    }
    
    fn create_unique_filepath(settings: &Settings, unique_filename: &str) -> String {
        format!("{}/{}", settings.tmp_path, unique_filename)
    }



}

fn modify_filepath(filepath: &str, modif: &str) -> String {
    let mut chunks = filepath.split(".").map(|s| Cow::Borrowed(s)).collect::<Vec<_>>();
    let target_idx = chunks.len() - 2;
    chunks[target_idx] += modif;
    chunks.join(".")
}

#[test]
fn extr_extension() {
    let filepath = "./tmp/asd.jpg";
    let modified = modify_filepath(filepath, "_abc");
    assert_eq!("./tmp/asd_abc.jpg", modified.as_str());
}