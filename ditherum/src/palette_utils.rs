use std::collections::HashMap;

use palette::{
    color_difference::EuclideanDistance, 
    Srgb
};

use serde::{
    Deserialize, 
    Serialize
};

// pub trait AllowedChannelType {}

// impl AllowedChannelType for u8 {}
// impl AllowedChannelType for f32 {}

/// A collection of RGB colors stored as `palette::Srgb<u8>`.
///
/// `PaletteRgb` is designed for color palette manipulation, including extracting
/// dominant colors from images and creating predefined palettes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaletteSrgb<T> {
    colors: Vec<Srgb<T>>
}

// TODO make it generic somehow
impl PaletteSrgb<f32> {
    pub fn find_closest(&self, random_color: Srgb<f32>) -> Srgb<f32> {
        assert!(!self.colors.is_empty());

        let result = self.colors.iter()
            .min_by_key(|c| c.distance_squared(random_color) as i32)
            .unwrap();
        *result
    }
}

impl<T> PaletteSrgb<T> {

    pub fn from_iter<I: IntoIterator<Item = palette::Srgb<T>>>(iter: I) -> Self {
        Self { colors: iter.into_iter().collect() }
    }
}


impl PaletteSrgb<u8> {
    pub const BLACK_N_WHITE_COLORS: [Srgb<u8>; 2] = [
        Srgb::new(0, 0, 0),
        Srgb::new(255, 255, 255),
    ];

    pub const PRIMARY_COLORS: [Srgb<u8>; 3] = [
        Srgb::new(255, 0, 0),
        Srgb::new(0, 255, 0),
        Srgb::new(0, 0, 255),
    ];

    /// Creates a `PaletteRgb` from any slice or reference to RGB values.
    ///
    /// # Parameters
    /// - `colors`: A slice or other `AsRef<[Srgb<u8>]>` collection of colors.
    ///
    /// # Returns
    /// A new `PaletteRgb` containing the provided colors.
    pub fn from_colors<C: AsRef<[Srgb<u8>]>>(colors: C) -> Self {
        Self { colors: colors.as_ref().to_vec() }
    }

    /// Extracts a color palette from an image, ordered by color frequency.
    ///
    /// # Parameters
    /// - `img`: An `RgbImage` from the `image` crate.
    /// - `colors_max`: Optional limit to the number of colors to include. If `None`, all unique colors are included.
    ///
    /// # Returns
    /// A `PaletteRgb` containing the most frequently used colors in the image.
    pub fn from_image(img: &image::RgbImage, colors_max: Option<usize>) -> Self {
        // Count occurrences of each color
        let colors_counts = img.enumerate_pixels()
            .map(|(_, _, color)| { color }) 
            .fold(HashMap::new(), |mut acc, color| {
                acc.entry(color).and_modify(|cnt| { *cnt += 1; }).or_insert(1);
                acc
            });

        // Sort colors by frequency (descending)
        let mut colors_ordered = {
            let mut colors = colors_counts.into_iter().collect::<Vec<_>>();
            colors.sort_by_key(|(_, cnt)| std::cmp::Reverse(*cnt));
            colors
        };

        // Limit to top N colors
        if let Some(colors_max) = colors_max {
            colors_ordered.truncate(colors_max);
        }

        // Convert from image::Rgb<u8> to palette::Srgb<u8>
        let colors_transformed = colors_ordered.into_iter()
            .map(|(c, _)| color_manip::rgb_u8_to_srgb_u8(c))
            .collect();

        Self { colors: colors_transformed }
    }

    pub fn find_closest(&self, random_color: Srgb<f32>) -> Srgb<u8> {
        assert!(!self.colors.is_empty());

        let result = self.colors.iter()
            .min_by_key(|c| c.into_format().distance_squared(random_color) as i32)
            .unwrap();
        *result
    }
    // Count colors ?
}

pub mod color_manip {
    use palette::Srgb;

    pub fn rgb_u8_to_srgb_float(src: &image::Rgb<u8>) -> palette::Srgb<f32> {
        rgb_u8_to_srgb_u8(src).into_format()
    }

    pub fn rgb_u8_to_srgb_u8(src: &image::Rgb<u8>) -> palette::Srgb<u8> {
        Srgb::new(src.0[0], src.0[1], src.0[2])
    }

    pub fn srgb_u8_to_rgb_u8(src: &palette::Srgb<u8>) -> image::Rgb<u8> {
        image::Rgb([src.red, src.green, src.blue])
    }

    pub fn srgb_add(left: &palette::Srgb, right: &palette::Srgb) -> palette::Srgb {
        palette::Srgb::new(
            left.red + right.red,
            left.green + right.green,
            left.blue + right.blue
        )
    }

    pub fn srgb_sub(left: &palette::Srgb, right: &palette::Srgb) -> palette::Srgb {
        palette::Srgb::new(
            left.red - right.red,
            left.green - right.green,
            left.blue - right.blue
        )
    }

    pub fn srgb_mul_scalar(left: &palette::Srgb, scalar: f32) -> palette::Srgb {
        palette::Srgb::new(
            left.red * scalar,
            left.green * scalar,
            left.blue * scalar
        )
    }

    pub fn mix_color_channel(
        mix_factor: f32, 
        from_value: u8,
        to_value: u8
    ) -> u8 {
        let mix_factor = mix_factor.clamp(0.0, 1.0);
        let mixed_value = (1.0 - mix_factor) * (from_value as f32) + mix_factor * (to_value as f32);
        mixed_value.round().clamp(0.0, 255.0) as u8 
    }

    pub fn mix_rgb_colors(
        mix_factor: f32, 
        from_color: image::Rgb<u8>,
        to_color: image::Rgb<u8>
    ) -> image::Rgb<u8> {
        image::Rgb([
            mix_color_channel(mix_factor, from_color[0], to_color[0]),
            mix_color_channel(mix_factor, from_color[1], to_color[1]),
            mix_color_channel(mix_factor, from_color[2], to_color[2])
        ])
    }
}

impl<T> AsRef<[Srgb<T>]> for PaletteSrgb<T> {
    fn as_ref(&self) -> &[Srgb<T>] {
        &self.colors
    }
}

impl From<&[Srgb<u8>]> for PaletteSrgb<u8>  {
    fn from(colors: &[Srgb<u8>]) -> Self {
        Self::from_colors(colors)
    }
}

impl<const N: usize> From<&[Srgb<u8>; N]> for PaletteSrgb<u8> {
    fn from(colors: &[Srgb<u8>; N]) -> Self {
        Self::from_colors(colors)
    }
}

impl From<&PaletteSrgb<u8>> for PaletteSrgb<f32> {
    fn from(palette: &PaletteSrgb<u8>) -> Self {
        let colors = palette.colors.iter()
            .map(|c| c.into_format())
            .collect();

        Self { colors }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image_utils::generate_gradient_image; // adjust module path to match your layout
    
    #[test]
    fn test_builtin_palettes_creations() {
        let palette_black_and_white = PaletteSrgb::from_colors(&PaletteSrgb::BLACK_N_WHITE_COLORS);
        assert_eq!(palette_black_and_white.as_ref().len(), 2);

        let palette_black_and_white = PaletteSrgb::from(PaletteSrgb::BLACK_N_WHITE_COLORS.as_slice());
        assert_eq!(palette_black_and_white.as_ref().len(), 2);

        let palette_primary = PaletteSrgb::from(&PaletteSrgb::PRIMARY_COLORS);
        assert_eq!(palette_primary.as_ref().len(), 3);
    }

    #[test]
    fn test_palette_from_gradient_image_2_colors_cap() {
        let cap = 2;
        let img = generate_gradient_image(
            100, 10,
            image::Rgb([0, 0, 0]),
            image::Rgb([255, 255, 255])
        );

        let palette = PaletteSrgb::from_image(&img, Some(cap));

        assert!(!palette.colors.is_empty());
        assert!(palette.colors.len() <= cap);
    }

    #[test]
    fn test_palette_from_gradient_image_no_colors_cap() {
        let expected_count = 16;
        let img = generate_gradient_image(
            expected_count as u32, 2,
            image::Rgb([0, 0, 0]),
            image::Rgb([255, 255, 255])
        );

        let palette = PaletteSrgb::from_image(&img, None);

        assert!(!palette.colors.is_empty());
        assert!(palette.colors.len() <= expected_count);
    }

    #[test]
    fn test_palette_closest_color() {
        let palette = PaletteSrgb::from_colors(&PaletteSrgb::BLACK_N_WHITE_COLORS);
        let closest_black = palette.find_closest(Srgb::new(1.3, 0.0, 0.0));
        let closest_white = palette.find_closest(Srgb::new(122.1, 0.0, 0.0));

        assert!(palette.as_ref().contains(&closest_black));
        assert!(palette.as_ref().contains(&closest_white));

        assert_eq!(closest_black, Srgb::new(0, 0, 0));
        assert_eq!(closest_white, Srgb::new(255, 255, 255));
    }
}