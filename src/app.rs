use std::path::PathBuf;

use diamonds_imager_generator::dmc::PaletteDmc;

#[derive(Clone)]
pub struct AppData {
    pub uplad_dir: PathBuf,
    pub image_max_width: usize,
    pub image_max_height: usize,
    pub dmc_full_palette: PaletteDmc,
}