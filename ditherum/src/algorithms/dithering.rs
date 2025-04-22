use image::RgbImage;
use crate::{color, palette::PaletteRGB};
use crate::algorithms::kernel;

/// Applies Floyd-Steinberg dithering to an RGB image using a given color palette.
///
/// # Parameters
/// - `source_image`: The input `RgbImage` to be dithered.
/// - `palette`: A `PaletteRGB` containing the target colors for dithering.
///
/// # Returns
/// - A dithered `RgbImage` that approximates the input image using the specified palette.
///
/// # Algorithm Details
/// The Floyd-Steinberg dithering algorithm works by replacing each pixel with the closest
/// color from the given palette and then distributing the quantization error to neighboring pixels
/// using a modified 2x2 kernel:
///
/// ```plaintext
///   (X)  *
///   *    *   (error distribution)
/// ```
pub fn dithering_floyd_steinberg_rgb(source_image: RgbImage, palette: PaletteRGB) -> RgbImage {
    let (width, height, mut rgb_matrix) = crate::image::manip::rgb_image_to_float_srgb_vec(source_image);
    let srgb_palette = palette.clone().to_srgb();

    kernel::apply_2x2_kernel_processing(&mut rgb_matrix, |kernel| {
        let closest_tl_color = color::manip::find_closest_srgb_color(kernel.tl , &srgb_palette);
        let quant_error = color::manip::srgb_sub(kernel.tl, &closest_tl_color);
        *kernel.tl = closest_tl_color;
    
        // Spread quantisation error over remaining 3 pixels
        // Keep errors weights low to prevent saturation
        let (err_weight_tr, err_weight_bl, err_weight_br) = (
            1.5 / 18.0,
            2.5 / 18.0,
            4.2 / 18.0,
        );
    
        *kernel.tr = color::manip::srgb_add(
            kernel.tr, 
            &color::manip::srgb_mul_scalar(&quant_error, err_weight_tr)
        );
        *kernel.bl = color::manip::srgb_add(
            kernel.bl, 
            &color::manip::srgb_mul_scalar(&quant_error, err_weight_bl)
        );
        *kernel.br = color::manip::srgb_add(
            kernel.br, 
            &color::manip::srgb_mul_scalar(&quant_error, err_weight_br)
        );
    });

    crate::image::manip::srgb_vec_to_rgb_image_using_palette(width, height, rgb_matrix, &palette)
}