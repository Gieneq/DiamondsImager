use crate::image_utils::{
    image_rgb_to_matrix_srgb_f32, 
    matrix_srgb_float_palette_quantization
};

use crate::palette_utils::color_manip::{
    srgb_add, 
    srgb_mul_scalar, 
    srgb_sub
};

use crate::palette_utils::PaletteSrgb;

use crate::algorithms::kernel;

pub fn dithering_floyd_steinberg_srgb(source_image: image::RgbImage, palette_srgb_u8: PaletteSrgb<u8>) -> image::RgbImage {
    let mut matrix_float_srgb = image_rgb_to_matrix_srgb_f32(&source_image);
    let palette_srgb_float = PaletteSrgb::<f32>::from(&palette_srgb_u8);

    kernel::apply_2x2_kernel_processing(&mut matrix_float_srgb, |kernel| {
        let closest_tl_color = palette_srgb_float.find_closest(*kernel.tl);
        let quant_error = srgb_sub(kernel.tl, &closest_tl_color);
        *kernel.tl = closest_tl_color;
        
        // Spread quantisation error over remaining 3 pixels
        // Keep errors weights low to prevent saturation
        let (err_weight_tr, err_weight_bl, err_weight_br) = (
            1.5 / 18.0,
            2.5 / 18.0,
            4.2 / 18.0,
        );
    
        *kernel.tr = srgb_add(
            kernel.tr, 
            &srgb_mul_scalar(&quant_error, err_weight_tr)
        );
        *kernel.bl = srgb_add(
            kernel.bl, 
            &srgb_mul_scalar(&quant_error, err_weight_bl)
        );
        *kernel.br = srgb_add(
            kernel.br, 
            &srgb_mul_scalar(&quant_error, err_weight_br)
        );
    });

    matrix_srgb_float_palette_quantization(&matrix_float_srgb, &palette_srgb_u8)
}