use image::RgbImage;

use crate::{color::ColorRGB, palette::PaletteRGB};
/// Applies thresholding to an image in RGB space by replacing each pixel with the closest color from the palette.
/// 
/// # Parameters
/// - `source_image`: The input `RgbImage` to be processed.
/// - `palette`: The color palette to use for thresholding.
/// 
/// # Returns
/// An `RgbImage` where each pixel is replaced by the closest color from the palette using RGB distance.
pub fn thresholding_rgb(mut source_image: RgbImage, palette: PaletteRGB) -> RgbImage {
    source_image.enumerate_pixels_mut()
        .for_each(|(_, _, pixel)| {
            *pixel = palette.find_closest_by_rgb(&ColorRGB::from_rgbu8(*pixel)).to_rgbu8()
        });

    source_image
}

/// Applies thresholding to an image in Lab space by replacing each pixel with the closest color from the palette.
/// 
/// # Parameters
/// - `source_image`: The input `RgbImage` to be processed.
/// - `palette`: The color palette to use for thresholding.
/// 
/// # Returns
/// An `RgbImage` where each pixel is replaced by the closest color from the palette using Lab color distance.
pub fn thresholding_lab(mut source_image: RgbImage, palette: PaletteRGB) -> RgbImage {
    source_image.enumerate_pixels_mut()
        .for_each(|(_, _, pixel)| {
            *pixel = palette.find_closest_by_lab(&ColorRGB::from_rgbu8(*pixel)).to_rgbu8()
        });

    source_image
}