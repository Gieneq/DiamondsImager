use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ExtractQueryMaxColorsCount {
    pub max_colors: Option<usize>,
}