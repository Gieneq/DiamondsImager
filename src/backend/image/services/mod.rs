pub mod errors;

use actix_multipart::form::tempfile::TempFile;
use actix_web::dev::Path;
use image::{Rgb, RgbImage};

use crate::settings::Settings;
use self::errors::InputFileError;


/// Functionality to save temporary file
pub fn save_file_with_generated_unique_filename(settings: &Settings, file: TempFile) -> Result<String, InputFileError> {
    let file = validate_input_file(settings, file)?;
    let unique_filename = create_unique_filename(file.file_name.as_ref().map(|s| s.as_str()))?;
    let unique_filepath = create_unique_filepath(settings, &unique_filename);
    log::info!("saving to '{unique_filepath}'");
   
    
    match file.file.persist_noclobber(unique_filepath.clone()) {
        Ok(_) => Ok(unique_filename),
        Err(e) => {
            log::error!("Could not save file, reason {e}");
            Err(InputFileError::FileNotSaved)
        }
    }
}

fn validate_input_file(settings: &Settings, file: TempFile) -> Result<TempFile, InputFileError> {
    if file.size == 0 {
        Err(InputFileError::FileEmpty)
    } else if file.size > settings.image_max_bytes{
        Err(InputFileError::FileTooBig { 
            actual: file.size, 
            limit: settings.image_max_bytes 
        })
    } else if file.file_name.is_none() {
        Err(InputFileError::FileNoname)
    } else {
        Ok(file)
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

#[test]
fn test_image_creation() {
    use tempfile::NamedTempFile;
    let settings = Settings::load();
    let empty_temp_file = TempFile {
        file: NamedTempFile::new().unwrap(),
        file_name: Some("test_file.jpg".to_string()),
        content_type: None,
        size: 0
    };

    let result = validate_input_file(&settings, empty_temp_file);
    assert!(matches!(result, Err(InputFileError::FileEmpty)));
}


fn calc_color_distance(color1: &[f32; 3], color2: &[f32; 3]) -> [f32; 3] {
    let dr = color1[0] - color2[0];
    let dg = color1[1] - color2[1];
    let db = color1[2] - color2[2];
    [dr, dg, db]
}

fn calc_color_distance_abs(color1: &[f32; 3], color2: &[f32; 3]) -> f32 {
    let [dr, dg, db] = calc_color_distance(color1, color2);
    (dr.powi(2) + dg.powi(2) + db.powi(2)).sqrt()
}

fn find_closest_from_pallet(src_color: &[f32; 3], float_pallet: &[[f32; 3]]) -> [f32; 3] {
    if float_pallet.is_empty() {
        panic!("Pallet empty");
    }

    let mut overall_min_dist = f32::MAX;
    let mut best_color_idx = 0;

    float_pallet.iter().enumerate().for_each(|(pallet_idx, pallet_color)| {
        let recent_dist = calc_color_distance_abs(src_color, pallet_color);
        if recent_dist < overall_min_dist {
            overall_min_dist = recent_dist;
            best_color_idx = pallet_idx;
        }
    });
    float_pallet[best_color_idx]
}

fn add_f32_3(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [ a[0] + b[0], a[1] + b[1], a[2] + b[2] ]
}

fn sub_f32_3(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [ a[0] - b[0], a[1] - b[1], a[2] - b[2] ]
}

fn len_f32_3(a: &[f32; 3]) -> f32 {
    (a[0].powi(2) + a[1].powi(2) + a[2].powi(2)).sqrt()
}

fn mul_f32_3(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [ a[0] * b[0], a[1] * b[1], a[2] * b[2] ]
}

fn mul_f32_3_scalar(a: &[f32; 3], b: f32) -> [f32; 3] {
    [ a[0] * b, a[1] * b, a[2] * b ]
}

fn constrain_f32_3(a: [f32; 3]) -> [f32; 3]  {
    [ a[0].min(255.0), a[1].min(255.0), a[2].min(255.0) ]
}

fn process_kernel_pixel(pixel: &[f32; 3], err: &[f32; 3], factor: f32) -> [f32; 3] {
    let weighted_err = mul_f32_3_scalar(err, factor);
    let summed = add_f32_3(pixel, &weighted_err);
    constrain_f32_3(summed)
}

fn process_kernel(input: &[&[f32; 3]], float_pallet: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let old_main_pixel = input.get(0).unwrap();
    let main_pixel = find_closest_from_pallet(input.get(0).unwrap(), float_pallet);
    let err = calc_color_distance(old_main_pixel, &main_pixel);
    let factors: [f32; 3] = [
        6.0 / 22.0,
        5.0 / 22.0,
        4.0 / 22.0
    ];
    // TODO add randomness

    //buffer[ny][nx][i] = (buffer[ny][nx][i] + error[i] * factor).clamp(0.0, 255.0);
    // Pass main as it is
    // add error mul by factor to remaining
    vec![
        main_pixel,
        process_kernel_pixel(input.get(1).unwrap(), &err, factors[0]),
        process_kernel_pixel(input.get(2).unwrap(), &err, factors[1]),
        process_kernel_pixel(input.get(3).unwrap(), &err, factors[2])
    ]
}

fn process_image(input_img: &RgbImage, pallette: &[Rgb<u8>]) -> RgbImage {
    let (width, height) = input_img.dimensions();
    let float_pallet = pallette.iter().map(|color| [
        color[0] as f32,
        color[1] as f32,
        color[2] as f32
    ]).collect::<Vec<_>>();

    // working buffer in floats
    let mut buffer: Vec<Vec<[f32; 3]>> = (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    let pixel = input_img.get_pixel(x, y);
                    [pixel[0] as f32, pixel[1] as f32, pixel[2] as f32]
                })
                .collect()
        })
        .collect();

    // process tmp buffer
    for y in 0..(height as usize - 1) {
        for x in 0..(width as usize - 1) {
            let kernel_indices = [
                (x, y), (x + 1, y),
                (x, y + 1), (x + 1, y + 1)
            ];

            let old_values = kernel_indices.iter()
                .map(|&(x, y)| buffer.get(y).unwrap().get(x).unwrap())
                .collect::<Vec<_>>();

            let new_values = process_kernel(&old_values, &float_pallet);
            new_values.iter()
                .zip(kernel_indices)
                .for_each(|(&new_value, (x, y))| {
                    *buffer.get_mut(y).unwrap().get_mut(x).unwrap() = new_value;
                });
        }
    }


    // back to RGB u8
    let mut result = RgbImage::new(width, height);
    for (y, row) in buffer.iter().enumerate() {
        for (x, raw_pixel) in row.iter().enumerate() {
            result.put_pixel(
                x as u32, 
                y as u32, 
                Rgb([
                    raw_pixel[0].round() as u8,
                    raw_pixel[1].round() as u8,
                    raw_pixel[2].round() as u8
                ])
            );
        }
    }

    result
}

#[test]
fn test_image_processing() {
    let absolute_img_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/res/test_ok_image.jpg");
    
    let absolute_dst_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tmpsf/test_ok_image_processed.jpg");

    println!("Opening image at absolute path={absolute_img_path:?}");
    let img = image::open(absolute_img_path).expect("Failed to open test image");
    
    let rgb_img = img.to_rgb8();
    let pallette = vec![
        Rgb([0,0,0]),

        Rgb([255,77,0]),
        Rgb([30,255,30]),
        Rgb([0,122,255]),
        Rgb([10,0,255]),

        Rgb([255,255,255]),
    ];

    let processed_imd = process_image(&rgb_img, &pallette);
    processed_imd.save(absolute_dst_path).expect("Failed to save processed test image");
}








// use std::borrow::Cow;

// pub fn get_hello_message() -> String {
//     "Hello".to_string()
// }

// pub mod processing {
//     use crate::{services::modify_filepath, settings::{Settings, Size}};
//     use actix_multipart::form::tempfile::TempFile;
//     use image::{imageops::FilterType, DynamicImage, GenericImage, GenericImageView, Pixel};
    


//     #[derive(Debug, Clone, Copy)]
//     pub struct Constrins {
//         pub diamons_size: f32,
//         pub result_size_mm: Size<f32>
//     }

//     impl Default for Constrins {
//         fn default() -> Self {
//             Self {
//                 diamons_size: 2.5,
//                 result_size_mm: Size {
//                     width: 500.0,
//                     height: 300.0 
//                 }
//             }
//         }
//     }

//     impl Constrins {
//         fn get_size_in_diamonds(&self) -> Size<u32> {
//             fn round_down(value: f32) -> u32 {
//                 value.floor() as u32
//             }

//             Size {
//                 width: round_down(self.result_size_mm.width / self.diamons_size),
//                 height: round_down(self.result_size_mm.height / self.diamons_size)
//             }
//         }
//     }

//     pub fn process_image(
//         filepath: String,
//         min_size: Size<u32>,
//         max_size: Size<u32>,
//         constrins: &Constrins
//     ) -> Result<String, ProcessingFileError> {
//         let image = image::open(filepath.clone()).map_err(|_e| {
//             ProcessingFileError::OtherError
//         })?;

//         log::info!("Validating image {}", filepath);
//         let image = validate_src_img_size(min_size, max_size, image)?;

//         log::info!("Processing image {}", filepath);
//         let processed_image = apply_processing(constrins, image);

//         let new_filepath = modify_filepath(&filepath, "_prev");

//         log::info!("Processing done! Got new image {}. Saving...", new_filepath);

//         //save image
//         processed_image.save(new_filepath.clone()).map_err(|_e| {
//             ProcessingFileError::OtherError
//         })?;

//         Ok(new_filepath)
//     }

//     fn validate_src_img_size(
//         min_size: Size<u32>,
//         max_size: Size<u32>,
//         image: DynamicImage
//     ) -> Result<DynamicImage, ProcessingFileError> {
//         if image.width() <= min_size.width {
//             Err(ProcessingFileError::WidthTooSmall { actual: image.width(), min: min_size.width })
//         }
//         else if image.width() > max_size.width {
//             Err(ProcessingFileError::WidthTooBig { actual: image.width(), max: min_size.width })
//         }
//         else if image.height() <= min_size.height {
//             Err(ProcessingFileError::HeightTooSmall { actual: image.height(), min: min_size.height })
//         }
//         else if image.height() > max_size.height {
//             Err(ProcessingFileError::HeightTooBig { actual: image.height(), max: max_size.height })
//         }
//         else { Ok(image) }
//     }

//     fn apply_processing(
//         constrins: &Constrins,
//         image: DynamicImage
//     ) -> DynamicImage {
//         log::info!("Constrins={:?}", constrins.get_size_in_diamonds());
//         let resized = image.resize(
//             constrins.get_size_in_diamonds().width,
//             constrins.get_size_in_diamonds().height,
//             FilterType::CatmullRom
//         );

//         let mut result = DynamicImage::new(resized.width(), resized.height(), image::ColorType::Rgb8);

//         for (x, y, color) in resized.pixels() {
//             let color = color.to_rgb();
//             result.put_pixel(x, y, color.to_rgba());
//         }

//         result
//     }
    


// }

// fn modify_filepath(filepath: &str, modif: &str) -> String {
//     let mut chunks = filepath.split(".").map(|s| Cow::Borrowed(s)).collect::<Vec<_>>();
//     let target_idx = chunks.len() - 2;
//     chunks[target_idx] += modif;
//     chunks.join(".")
// }

// #[test]
// fn extr_extension() {
//     let filepath = "./tmp/asd.jpg";
//     let modified = modify_filepath(filepath, "_abc");
//     assert_eq!("./tmp/asd_abc.jpg", modified.as_str());
// }