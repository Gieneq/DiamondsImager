use crate::palette_utils::{self, color_manip, PaletteSrgb};

/// Generates a horizontal gradient image.
/// 
/// # Parameters
/// - `width`: Image width.
/// - `height`: Image height.
/// - `from_color`: Starting color.
/// - `to_color`: Ending color.
/// 
/// # Returns
/// A generated `image::RgbImage` with a color gradient.
pub fn generate_gradient_image(
    width: u32, 
    height: u32,
    from_color: image::Rgb<u8>,
    to_color: image::Rgb<u8>
) -> image::RgbImage {
    assert!(width > 0, "Width should be > 0");
    assert!(height > 0, "Height should be > 0");

    let mut img = image::RgbImage::new(width, height);

    for x in 0..width {
        let mix_factor = (x as f32) / (width - 1) as f32;
        let pixel_color = super::color::manip::mix_rgb_colors(mix_factor, from_color, to_color);
        (0..height).for_each(|y| {
            *img.get_pixel_mut(x, y) = pixel_color;
        });
    }

    img
}

/// Converts an `image::RgbImage` to a 2D vector of `palette::Srgb<f32>`.
pub fn image_rgb_to_matrix_srgb_f32(source_image: &image::RgbImage) -> Vec<Vec<palette::Srgb<f32>>> {
    let (width, height) = (source_image.width() as usize, source_image.height() as usize);

    let mut srgb_float_image = vec![vec![palette::Srgb::new(0.0, 0.0, 0.0); width]; height];
    
    source_image.enumerate_pixels()
        .for_each(|(x, y, rgb_pixel)| {
            srgb_float_image[y as usize][x as usize] = color_manip::rgb_u8_to_srgb_float(rgb_pixel)
        });
    srgb_float_image
}

pub fn matrix_srgb_float_palette_quantization(matrix: &[Vec<palette::Srgb<f32>>], palette_srgb_u8: &PaletteSrgb<u8>) -> image::RgbImage {
    let height = matrix.len();
    assert!(height > 0);

    let width = matrix[0].len();
    assert!(width > 0);

    image::RgbImage::from_fn(width as u32, height as u32, |x, y| {
        let srgb_float_color = matrix[y as usize][x as usize];
        let srgb_u8_color = palette_srgb_u8.find_closest(srgb_float_color);
        palette_utils::color_manip::srgb_u8_to_rgb_u8(&srgb_u8_color)
    })
}