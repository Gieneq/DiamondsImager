use std::{collections::HashMap, path::Path};

use image::{ImageResult, RgbImage};

use crate::{algorithms::{dithering, thresholding}, palette::PaletteRGB};

/// Defines different image processing algorithms.
#[derive(Debug)]
pub enum ProcessingAlgorithm {
    ThresholdingRgb,
    ThresholdingLab,
    FloydSteinbergRgb,
}

/// Represents an image processor that applies a specified algorithm to an image.
#[derive(Debug)]
pub struct ImageProcessor {
    source_image: RgbImage,
    palette: PaletteRGB,
    algorithm: ProcessingAlgorithm,
}

/// Loads an image from a given file path.
/// 
/// # Parameters
/// - `path`: Path to the image file.
/// 
/// # Returns
/// A `Result` containing the loaded `RgbImage` or an error.
pub fn load_image<P>(path: P) -> ImageResult<RgbImage> 
where 
    P: AsRef<Path>
{
    let img = image::open(path)?;
    Ok(img.to_rgb8())
}

/// Saves an `RgbImage` to the specified file path.
/// 
/// # Parameters
/// - `path`: Destination file path.
/// - `img`: Reference to the image to be saved.
/// 
/// # Returns
/// A `Result` indicating success or failure.
pub fn save_image<P>(path: P, img: &RgbImage) -> ImageResult<()>
where 
    P: AsRef<Path>
{
    img.save(path)
}

/// Generates a horizontal gradient image.
/// 
/// # Parameters
/// - `width`: Image width.
/// - `height`: Image height.
/// - `from_color`: Starting color.
/// - `to_color`: Ending color.
/// 
/// # Returns
/// A generated `RgbImage` with a color gradient.
pub fn generate_test_gradient_image(
    width: u32, 
    height: u32,
    from_color: image::Rgb<u8>,
    to_color: image::Rgb<u8>
) -> RgbImage {
    if width == 0 {
        panic!("Width should be > 0");
    }

    if height == 0 {
        panic!("Height should be > 0");
    }

    let mut img = RgbImage::new(width, height);

    for x in 0..width {
        let mix_factor = (x as f32) / (width - 1) as f32;
        let pixel_color = super::color::manip::mix_rgb_colors(mix_factor, from_color, to_color);
        (0..height).for_each(|y| {
            *img.get_pixel_mut(x, y) = pixel_color;
        });
    }

    img
}

pub fn count_image_colors(src_img: &image::RgbImage) -> HashMap<image::Rgb<u8>, usize> {
    src_img.enumerate_pixels()
        .map(|(_, _, px)| px)
        .fold(HashMap::new(), |mut acc, px| {
            acc.entry(*px).and_modify(|count| *count += 1).or_insert(1);
            acc
        })
}

impl ImageProcessor {
    /// Creates a new `ImageProcessor` instance with a given image and palette.
    pub fn new(source_image: RgbImage, palette: PaletteRGB) -> Self {
        Self {
            source_image,
            palette,
            algorithm: ProcessingAlgorithm::ThresholdingRgb
        }
    }

    /// Sets the processing algorithm.
    pub fn with_algorithm(mut self, algorithm: ProcessingAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Executes the selected algorithm and processes the image.
    pub fn run(self) -> RgbImage {
        match self.algorithm {
            ProcessingAlgorithm::ThresholdingRgb => thresholding::thresholding_rgb(self.source_image, self.palette),
            ProcessingAlgorithm::ThresholdingLab => thresholding::thresholding_lab(self.source_image, self.palette),
            ProcessingAlgorithm::FloydSteinbergRgb => dithering::dithering_floyd_steinberg_rgb(self.source_image, self.palette),
        }
    }
}

pub mod manip {
    use image::DynamicImage;
    use palette::white_point::D65;

    use crate::color;

    use super::*;
    
    /// Converts an `RgbImage` to a 2D vector of `palette::Srgb`.
    pub fn rgb_image_to_float_srgb_vec(source_image: RgbImage) -> (usize, usize, Vec<Vec<palette::Srgb>>) {
        let (width, height) = (source_image.width() as usize, source_image.height() as usize);
        let mut lab_image = vec![vec![palette::Srgb::new(0.0, 0.0, 0.0); width]; height];
        
        source_image.enumerate_pixels()
            .for_each(|(x, y, rgb_pixel)| {
                lab_image[y as usize][x as usize] = color::manip::rgbu8_to_srgb(*rgb_pixel)
            });

        (width, height, lab_image)
    }

    /// Converts an `RgbImage` to a 2D vector of `palette::Lab<D65, f32>`.
    pub fn rgb_image_to_lab_vec(source_image: RgbImage) -> (usize, usize, Vec<Vec<palette::Lab<D65,f32>>>) {
        let (width, height) = (source_image.width() as usize, source_image.height() as usize);
        let mut lab_image = vec![vec![palette::Lab::new(0.0, 0.0, 0.0); width]; height];
        
        source_image.enumerate_pixels()
            .for_each(|(x, y, rgb_pixel)| {
                lab_image[y as usize][x as usize] = color::manip::rgbu8_to_lab(*rgb_pixel)
            });

        (width, height, lab_image)
    }

    /// Converts a 2D vector of `palette::Lab` to an `RgbImage`.
    pub fn lab_vec_to_rgb_image(width: usize, height: usize, lab_vec: Vec<Vec<palette::Lab>>) -> RgbImage {
        RgbImage::from_fn(width as u32, height as u32, |x, y| {
            let lab_color = &lab_vec[y as usize][x as usize];
            color::manip::lab_to_rgbu8(*lab_color)
        })
    }

    /// Converts a 2D vector of `palette::Srgb` to an `RgbImage`.
    pub fn srgb_vec_to_rgb_image(width: usize, height: usize, rgb_vec: Vec<Vec<palette::Srgb>>) -> RgbImage {
        RgbImage::from_fn(width as u32, height as u32, |x, y| {
            let srgb_color = &rgb_vec[y as usize][x as usize];
            color::manip::srgb_to_rgbu8(*srgb_color)
        })
    }

    /// Converts a 2D vector of `palette::Srgb` to an `RgbImage` ensuring palette coherency.
    pub fn srgb_vec_to_rgb_image_using_palette(width: usize, height: usize, rgb_vec: Vec<Vec<palette::Srgb>>, palette: &PaletteRGB) -> RgbImage {
        RgbImage::from_fn(width as u32, height as u32, |x, y| {
            let srgb_color = &rgb_vec[y as usize][x as usize];
            palette.find_closest_by_srgb(srgb_color).into()
        })
    }

    /// Converts an `RgbImage` to a new size while preserving aspect ratio.
    pub fn rgb_image_reshape(src_img: RgbImage, width: Option<u32>, height: Option<u32>) -> RgbImage {
        let dyn_img = DynamicImage::from(src_img);

        let (original_width, original_height) = (dyn_img.width(), dyn_img.height());
        let (new_width, new_height) = match (width, height) {
            (Some(w), Some(h)) => (w, h),
            (None, None) => (original_width, original_height),
            (None, Some(h)) => {
                let w = (h as f32 * original_width as f32 / original_height as f32).round() as u32;
                (w, h)
            },
            (Some(w), None) => {
                let h = (w as f32 * original_height as f32 / original_width as f32).round() as u32;
                (w, h)
            },
        };

        dyn_img.resize_to_fill(
            new_width, 
            new_height, 
            image::imageops::FilterType::Lanczos3
        ).into()
    }
}

#[test]
fn test_processing_gradient_image() {
    let (width, height) = (200, 80);
    let source_image = generate_test_gradient_image(
        width, 
        height, 
        image::Rgb::<u8>([0,0,0]), 
        image::Rgb::<u8>([0,0,255]), 
    );
    let palette = PaletteRGB::primary();

    let processing_result = ImageProcessor::new(source_image, palette)
        .run();
    assert_eq!(processing_result.width(), width);
    assert_eq!(processing_result.height(), height);
}