use std::{
    collections::{HashMap, HashSet}, 
    fs::File, 
    io::{
        BufReader, 
        BufWriter
    }, ops::{
        Deref, 
        DerefMut
    }, 
    path::Path, 
    vec
};
use errors::PaletteError;
use palette::color_difference::{
    Ciede2000, 
    EuclideanDistance
};
use serde::{
    Serialize, 
    Deserialize
};
use crate::{
    algorithms::kmean, 
    color::{
        self, 
        ColorRGB
    }
};

pub mod errors {
    use crate::algorithms::kmean::CentroidsFindError;

    #[derive(Debug, thiserror::Error)]
    pub enum PaletteError {
        #[error("Not enough colors to be converted to: {0}.")]
        NotEnoughColors(usize),

        #[error("Faild to convert, reason={0}")]
        ConvertionErrot(CentroidsFindError),

        #[error("I/O error, reason={0}")]
        IoError(std::io::Error),

        #[error("JSON parsing failed, reason={0}")]
        JsonParsingFailed(serde_json::error::Error),

        #[error("PaletteEmpty")]
        PaletteEmpty,
    }

    impl From<CentroidsFindError> for PaletteError {
        fn from(value: CentroidsFindError) -> Self {
            Self::ConvertionErrot(value)
        }
    }

    impl From<std::io::Error> for PaletteError {
        fn from(value: std::io::Error) -> Self {
            Self::IoError(value)
        }
    }

    impl From<serde_json::error::Error> for PaletteError {
        fn from(value: serde_json::error::Error) -> Self {
            Self::JsonParsingFailed(value)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PaletteRGB(Vec<ColorRGB>);

impl PaletteRGB {
    
    /// Extracts a palette from an image by collecting unique pixel colors.
    pub fn from_rgbu8_image(img: &image::RgbImage) -> Self {
        let mut palette_set = HashSet::new();

        for y in 0..img.height() {
            for x in 0..img.width() {
                let pixel = img.get_pixel(x, y);
                palette_set.insert(*pixel);
            }
        }

        // Sorting included
        Self::from(palette_set)
    }

    /// Returns a palette containing only black and white.
    pub fn black_and_white() -> Self {
        PaletteRGB::from(vec![
            ColorRGB([0, 0, 0]),
            ColorRGB([255, 255, 255]),
        ])
    }

    /// Returns a palette of primary colors: red, green, and blue.
    pub fn primary() -> Self {
        PaletteRGB::from(vec![
            ColorRGB([255, 0, 0]),
            ColorRGB([0, 255, 0]),
            ColorRGB([0, 0, 255]),
        ])
    }

    /// Returns a palette of colors: black, white, red, green, and blue.
    pub fn primary_bw() -> Self {
        PaletteRGB::from(vec![
            ColorRGB([0,   0, 0]),
            ColorRGB([255, 0, 0]),
            ColorRGB([0, 255, 0]),
            ColorRGB([0, 0, 255]),
            ColorRGB([255, 255, 255]),
        ])
    }

    /// Returns a grayscale palette with the specified number of steps.
    ///
    /// # Example
    ///
    /// ```
    /// use ditherum::palette::PaletteRGB;
    /// 
    /// let palette = PaletteRGB::grayscale(5);
    /// 
    /// println!("{palette:?}");
    /// // Produces: [black, dark gray, medium gray, light gray, white]
    /// ```
    pub fn grayscale(steps: usize) -> PaletteRGB {
        assert!(steps >= 2, "Grayscale palette requires at least two steps.");

        let colors = (0..steps)
            .map(|step| {
                let channel_value = ((255 * step) / (steps - 1)) as u8;
                ColorRGB([channel_value, channel_value, channel_value])
            })
            .collect::<Vec<_>>();

        PaletteRGB(colors)
    }

    pub fn with_black_and_white(mut self) -> Self {
        self.combine(Self::black_and_white());
        self
    }

    /// Attempts to reduce the number of colors in the palette to a specified target count.
    ///
    /// This method is useful when you want to simplify a color palette by reducing the number
    /// of distinct colors while preserving the overall color harmony as much as possible. It 
    /// uses a clustering technique to find the best fitting centroids that represent the reduced 
    /// color set.
    ///
    /// # Parameters
    /// - `target_colors_count`: The desired number of colors in the reduced palette.
    ///
    /// # Returns
    /// - `Ok(Self)`: If the palette was successfully reduced to the target number of colors.
    /// - `Err(PaletteError::NotEnoughColors)`: If the requested number of colors is greater than 
    ///   the current number of colors in the palette.
    ///
    /// # Errors
    /// - `PaletteError::NotEnoughColors`: Returned when the requested number of colors is greater 
    ///   than the available number of colors in the palette.
    ///
    /// # Panics
    /// This method does not panic.
    ///
    /// # Example
    /// ```
    /// use ditherum::palette::PaletteRGB;
    /// 
    /// let palette = PaletteRGB::primary();
    ///
    /// let reduced_palette = palette.try_reduce(2).expect("Failed to reduce colors");
    /// println!("{:?}", reduced_palette);
    /// ```
    ///
    /// In this example, the palette is reduced to 2 colors while maintaining the color balance
    /// using a clustering algorithm to find the best fitting centroids.
    pub fn try_reduce(self, target_colors_count: usize) -> Result<Self, self::errors::PaletteError> {
        match self.len().cmp(&target_colors_count) {

            // Cannot obtain bigger pallete than the input pallet size
            std::cmp::Ordering::Less => Err(self::errors::PaletteError::NotEnoughColors(self.len())),

            // Te same pallet
            std::cmp::Ordering::Equal => Ok(self),

            // Reduce colors count
            std::cmp::Ordering::Greater => {

                let lab_colors: Vec<palette::Lab> = self.into();

                // Apply clusterization to find best fitting centroids
                let new_lab_colors = find_lab_colors_centroids(
                    &lab_colors, 
                    target_colors_count
                )?;
                let mut palette = PaletteRGB::from(new_lab_colors);
                palette.sort();
                Ok(palette)
            },
        }
    }

    /// Attempts to find a subset of the current palette that best matches the image content.
    /// 
    /// This is useful when the palette contains more colors than needed, and you'd like to reduce
    /// it to a representative subset (e.g., for color quantization or palette-based compression).
    /// 
    /// It works by mapping each pixel in the provided image to the closest color from the current
    /// palette, counting how frequently each palette color appears, and selecting the `max_colors_count`
    /// most common colors.
    /// 
    /// # Arguments
    /// - `max_colors_count`: Maximum number of colors to keep in the resulting palette.
    /// - `raw_image`: An RGB image to extract color usage from.
    /// 
    /// # Returns
    /// - `Ok(PaletteRGB)`: A new palette containing the most frequently used colors from the original palette.
    /// - `Err(PaletteError::NotEnoughColors)`: If the palette contains fewer colors than requested.
    /// 
    /// ```
    pub fn try_find_closest_subset_using_image(
        self, 
        max_colors_count: usize, 
        raw_image: &image::RgbImage
    ) -> Result<Self, self::errors::PaletteError> {
        // Cannot obtain a larger palette than the one we have
        if self.len() < max_colors_count {
                return Err(self::errors::PaletteError::NotEnoughColors(self.len()));
        }

    // Map each pixel in the image to the closest color in the current palette
        let mapped_to_palette_colors = raw_image
            .pixels()
            .map(|px| {
                let px_color = ColorRGB::from_rgbu8(*px);
                self.find_closest_by_rgb(&px_color)
            })
            .collect::<Vec<_>>();

        // Count the frequency of each palette color
        let mapped_colors_counts: HashMap<ColorRGB, usize> = mapped_to_palette_colors.iter()
            .fold(HashMap::new(), |mut acc, c| {
                acc.entry(*c).and_modify(|cnt| *cnt += 1).or_insert(1);
                acc
            });
        let mut found_colors = mapped_colors_counts.into_iter().collect::<Vec<_>>();

        // Find expected colors count
        let expected_colors_count = max_colors_count.min(found_colors.len());

        // Find most common colors
        found_colors.sort_by_key(|(_, cnt)| -(*cnt as isize));
        let most_common_colors = &found_colors[..expected_colors_count];
        
        let tmp_colors_vec = most_common_colors.iter()
            .map(|(c, _)| *c)
            .collect::<Vec<_>>();

        Ok(Self::from(tmp_colors_vec))
    }

    /// Saves the palette to a JSON file at the specified path.
    ///
    /// # Parameters
    /// - `path`: The file path where the JSON data should be saved.
    ///
    /// # Errors
    /// - Returns an `io::Error` if there is an issue creating or writing to the file.
    ///
    /// # Example
    /// ```
    /// use ditherum::palette::PaletteRGB;
    /// 
    /// let palette = PaletteRGB::primary();
    /// 
    /// palette.save_to_json("tmp_palette.json").expect("Failed to save palette");
    /// ```
    pub fn save_to_json<P>(&self, path: P) -> Result<(), PaletteError> 
    where 
        P: AsRef<Path>
    {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
    
    /// Loads the palette from a JSON file at the specified path.
    ///
    /// # Parameters
    /// - `path`: The file path from which to read the JSON data.
    ///
    /// # Returns
    /// - `Ok(PaletteRGB)`: If the JSON data is successfully parsed into a `PaletteRGB`.
    /// - `Err(io::Error)`: If there is an issue reading the file.
    /// - `Err(serde_json::Error)`: If there is an issue parsing the JSON data.
    ///
    /// # Example
    /// ```
    /// use ditherum::palette::PaletteRGB;
    /// 
    /// let palette = PaletteRGB::load_from_json("tmp_palette.json").expect("Failed to load palette");
    /// println!("{:?}", palette);
    /// ```
    pub fn load_from_json<P>(path: P) -> Result<Self, PaletteError> 
    where 
        P: AsRef<Path>
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut pallete: PaletteRGB = serde_json::from_reader(reader)?;
        pallete.sort();
        Ok(pallete)
    }
    /// Generates a visualization of the ANSI colors in the palette.
    /// 
    /// This method converts each color in the palette to an ANSI background color block,
    /// followed by the color's RGB representation.
    /// 
    /// # Example
    /// ```
    /// use ditherum::palette::PaletteRGB;
    /// 
    /// let palette = PaletteRGB::primary();
    /// let visualization = palette.get_ansi_colors_visualization();
    /// println!("{visualization}");
    /// 
    /// // This would print:
    /// // █ : (255, 0, 0)
    /// // █ : (0, 255, 0)
    /// // █ : (0, 0, 255)
    /// // Each color block represents the corresponding RGB value.
    /// ```
    /// # Returns
    /// - A `String` containing the ANSI color visualization.
    /// - Returns an empty string if the palette is empty.
    /// 
    /// # Notes
    /// - This uses True Color (24-bit) ANSI escape codes, so it requires a terminal
    ///   that supports True Color (most modern terminals do).
    /// - If your terminal doesn't support True Color, the colors may not display correctly.
    /// 
    /// # See Also
    /// - [ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code)
    pub fn get_ansi_colors_visualization(&self) -> String {
        // Empty self -> unwrap to default = empty sttring
        self.iter()
            .map(|color| {
                let (r, g, b) = color.tuple();
                format!("\x1b[48;2;{};{};{}m  \x1b[0m: {:?}\n", r, g, b, color.0)
            })
            .reduce(|mut acc, line| {
                acc += &line;
                acc
            })
            .unwrap_or_default()
    }

    /// Converts the palette to a vector of `image::Rgb<u8>`.
    /// 
    /// # Returns
    /// 
    /// A `Vec<image::Rgb<u8>>` representing the colors.
    pub fn to_rgbu8(self) -> Vec<image::Rgb<u8>> {
        self.into()
    }

    /// Converts the palette to a vector of `palette::Srgb`.
    /// 
    /// # Returns
    /// 
    /// A `Vec<palette::Srgb>` representing the colors.
    pub fn to_srgb(self) -> Vec<palette::Srgb> {
        self.into()
    }

    /// Converts the palette to a vector of `palette::Lab`.
    /// 
    /// # Returns
    /// 
    /// A `Vec<palette::Lab>` representing the colors.
    pub fn to_lab(self) -> Vec<palette::Lab> {
        self.into()
    }

    /// Finds the closest color in the palette to the given color using Lab distance.
    /// 
    /// # Parameters
    /// 
    /// - `src_color`: The reference color.
    /// 
    /// # Returns
    /// 
    /// The closest `ColorRGB` in the palette.
    pub fn find_closest_by_lab(&self, src_color: &ColorRGB) -> ColorRGB {
        let (_, &color) = self.iter()
            .map(|palette_color| (src_color.dist_by_lab(palette_color), palette_color))
            .min_by(|(diff_a, _), (diff_b, _)| diff_a.partial_cmp(diff_b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        color
    }

    /// Finds the closest color in the palette to the given color using RGB squared distance.
    /// 
    /// # Parameters
    /// 
    /// - `src_color`: The reference color.
    /// 
    /// # Returns
    /// 
    /// The closest `ColorRGB` in the palette.
    pub fn find_closest_by_rgb(&self, src_color: &ColorRGB) -> ColorRGB {
        let (_, &color) = self.iter()
            .map(|palette_color| (src_color.dist_squared_by_rgb(palette_color), palette_color))
            .min_by(|(diff_a, _), (diff_b, _)| diff_a.partial_cmp(diff_b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        color
    }

    /// Finds the closest color in the palette to the given color using Srgb squared distance.
    /// 
    /// # Parameters
    /// 
    /// - `src_color`: The reference `palette::Srgb` color.
    /// 
    /// # Returns
    /// 
    /// The closest `ColorRGB` in the palette.
    pub fn find_closest_by_srgb(&self, src_color: &palette::Srgb) -> ColorRGB {
        let (_, &color) = self.iter()
        .map(|palette_color| (src_color.distance_squared(palette_color.to_srgb()), palette_color))
        .min_by(|(diff_a, _), (diff_b, _)| diff_a.partial_cmp(diff_b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    color
    }

    /// Combines another palette into this one, removes duplicates, and sorts it.
    /// 
    /// # Parameters
    /// 
    /// - `other`: Another `PaletteRGB` to merge.
    pub fn combine(&mut self, mut other: Self) {
        self.append(&mut other);
        self.dedup();
        self.sort();
    }
}

/// Implements conversion from `PaletteRGB` to a vector of any type that can be converted from `ColorRGB`.
impl<T> From<PaletteRGB> for Vec<T> 
where 
    T: From<ColorRGB>
{
    fn from(value: PaletteRGB) -> Self {
        value.0.into_iter()
            .map(|v| T::from(v))
            .collect()
    }
}

/// Implements conversion from a reference to `PaletteRGB` to a vector of any type that can be converted from `ColorRGB`.
impl<T> From<&PaletteRGB> for Vec<T>
where 
    T: From<ColorRGB>,
{
    fn from(value: &PaletteRGB) -> Self {
        value.0.iter()
            .map(|&v| T::from(v))
            .collect()
    }
}

/// Implements conversion from a `HashSet<T>` to `PaletteRGB`, ensuring uniqueness.
impl<T> From<HashSet<T>> for PaletteRGB 
where 
    T: Into<ColorRGB>
{
    fn from(value: HashSet<T>) -> Self {
        let mut result = Self(value.into_iter()
            .map(|v| v.into())
            .collect()
        );
        result.sort();
        result
    }
}

/// Implements conversion from a `Vec<T>` to `PaletteRGB`, ensuring uniqueness.
impl<T> From<Vec<T>> for PaletteRGB 
where 
    T: Into<ColorRGB>
{
    fn from(value: Vec<T>) -> Self {
        let unique_colors: HashSet<ColorRGB> = value.into_iter().map(Into::into).collect();
        let mut result = Self(unique_colors.into_iter().collect());
        result.sort();
        result
    }
}

/// Allows treating `PaletteRGB` as a vector of `ColorRGB`.
impl Deref for PaletteRGB {
    type Target = Vec<ColorRGB>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Allows treating `PaletteRGB` as a mutable vector of `ColorRGB`.
impl DerefMut for PaletteRGB {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


/// Clusters Lab colors using k-means and returns new centroids.
/// 
/// # Parameters
/// 
/// - `input`: A slice of Lab colors.
/// - `centroids_count`: Number of centroids to compute.
/// 
/// # Returns
/// 
/// A `Result` containing new Lab centroids or an error if clustering fails.
fn find_lab_colors_centroids(
    input: &[palette::Lab], 
    centroids_count: usize
) -> Result<Vec<palette::Lab>, kmean::CentroidsFindError> {
    let lab_distance_measure = |a: &palette::Lab, b: &palette::Lab| {
        a.difference(*b)
    };

    let calculate_lab_mean = |arr: &[palette::Lab]| {
        let mut accumulator = arr.iter()
            .fold(palette::Lab::new(0.0, 0.0, 0.0), |mut acc, item| {
                color::manip::lab_mut_add(&mut acc, item);
                acc
            });
        accumulator.l /= arr.len() as f32;
        accumulator.a /= arr.len() as f32;
        accumulator.b /= arr.len() as f32;
        accumulator
    };

    kmean::find_centroids(
        input, 
        centroids_count, 
        lab_distance_measure, 
        calculate_lab_mean
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grayscale_palette() {
        let steps = 113;
        let palette = PaletteRGB::grayscale(steps);
        assert_eq!(palette.len(), steps);

        // Check endpoints are black and white.
        assert_eq!(palette[0], ColorRGB([0, 0, 0]));
        assert_eq!(palette[steps - 1], ColorRGB([255, 255, 255]));
    }

    #[test]
    fn test_try_reduce_not_enough_colors() {
        // Create a palette with only three colors.
        let palette = PaletteRGB::primary();

        // Trying to reduce to 4 colors should fail.
        let result = palette.clone().try_reduce(4);
        assert!(result.is_err());

        if let Err(errors::PaletteError::NotEnoughColors(actual)) = result {
            assert_eq!(actual, palette.len());
        } else {
            panic!("Expected NotEnoughColors error.");
        }
    }

    #[test]
    fn test_reduce_bn_w_palette() {
        let palette = PaletteRGB::black_and_white();
        assert_eq!(palette.len(), 2);

        let reduced_palette = palette.try_reduce(1);
        assert!(reduced_palette.is_ok());
        let reduced_palette = reduced_palette.unwrap();
        let reduced_color = reduced_palette[0];
        assert_eq!(reduced_color, ColorRGB([119, 119, 119]));
    }

    #[test]
    fn test_convertion_to_lab_and_from() {
        let test_palette = PaletteRGB::primary_bw();
        let lab_colors: Vec<palette::Lab> = (&test_palette).into();
        let recreated_palette = PaletteRGB::from(lab_colors);
        assert_eq!(test_palette, recreated_palette);
    }

    #[test]
    fn test_combining_palettes() {
        let bw_palette = PaletteRGB::black_and_white();
        let mut primary_palette = PaletteRGB::primary();
        primary_palette.combine(bw_palette);
        let combined_palette = primary_palette;

        let expected_combined_palette = PaletteRGB::primary_bw();
        assert_eq!(combined_palette, expected_combined_palette)

    }
}