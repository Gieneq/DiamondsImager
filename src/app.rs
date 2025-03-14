use std::path::PathBuf;

#[derive(Clone)]
pub struct AppData {
    pub uplad_dir: PathBuf,
    pub image_max_width: usize,
    pub image_max_height: usize,
}