use ditherum::algorithms::dithering::dithering_floyd_steinberg_srgb;

use crate::services::dmc::{
    DmcBom, 
    PaletteDmc
};

/// Dither the source image using a DMC palette and produce a BOM.
///
/// This function first converts the given `PaletteDmc` into an sRGB palette,
/// then applies Floyd–Steinberg dithering to the `src_img`, and finally
/// computes a Bill of Materials (BOM) mapping each DMC color to the count
/// of pixels using that color in the dithered image.
///
/// # Arguments
///
/// * `palette_dmc` – Reference to the `PaletteDmc` to use for palette lookup.
/// * `src_img` – The source `RgbImage` to which dithering will be applied.
///
/// # Returns
///
/// A tuple containing:
/// 1. The dithered `RgbImage`.
/// 2. A `DmcBom` (i.e., `HashMap<Dmc,u32>`) where each key is a `Dmc`
///    color present in the image and each value is the number of pixels
///    in the dithered image that use that color.
///
/// # Panics (in debug builds) but should not
///
/// This function includes a `debug_assert_eq!` to verify that every pixel
/// in the dithered image maps back to a color in the original DMC palette.
/// If any unmapped colors remain, it will panic in non-optimized builds.
pub fn image_dither_using_dmc_palette(palette_dmc: &PaletteDmc, src_img: &image::RgbImage) -> (image::RgbImage, DmcBom) {
    let palette_srgb = palette_dmc.downgrade_to_srgb_palette();
    let dithered_image = dithering_floyd_steinberg_srgb(src_img, &palette_srgb);

    let (dmc_bom, not_mapped_count) = palette_dmc.find_bom_of_image(&dithered_image);
    debug_assert_eq!(not_mapped_count, 0,  "dithered image contained colors outside the DMC palette, found {not_mapped_count} colors.");

    (dithered_image, dmc_bom)
}