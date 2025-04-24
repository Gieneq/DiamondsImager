use std::{collections::HashSet, fmt::Debug, hash::Hash, io::BufReader, path::Path};

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, thiserror::Error)]
pub enum DmcError {
    #[error("Io error, reason: {0}")]
    IoError(#[from] std::io::Error),

    #[error("serde_json Io error, reason: {0}")]
    SerdeJsonError(#[from] serde_json::error::Error),

    #[error("Faled to parsecolor hex: {0}")]
    HexColorParseFailed(String),

    #[error("Dmc data corrupted")]
    DmcDataCorrupted,

    #[error("Faled to parse int in hex color: {0}")]
    HexColorParseIntFailed(#[from] std::num::ParseIntError),

    #[error("Data in DMC palette is not unique")]
    DmcDataNotUnique,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Dmc {
    pub name: String,
    pub code: String,
    pub color: palette::Srgb<u8>,
}

impl Hash for Dmc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.code.hash(state);
        self.color.red.hash(state);
        self.color.green.hash(state);
        self.color.blue.hash(state);
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PaletteDmc {
    elements: HashSet<Dmc>
}

impl AsRef<HashSet<Dmc> > for PaletteDmc {
    fn as_ref(&self) -> &HashSet<Dmc> {
        &self.elements
    }
}

mod io {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
    pub struct DmcDataIo {
        pub name: String,
        pub code: String,
        pub color: String,
    }
    
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct PaletteDmcDataIo(pub HashSet<DmcDataIo>);

    impl From<super::Dmc> for DmcDataIo {
        fn from(value: super::Dmc) -> Self {
            let colorhash = format!("#{:02X}{:02X}{:02X}",
                value.color.red,
                value.color.green,
                value.color.blue,
            );
            Self {
                code: value.code,
                name: value.name,
                color: colorhash
            }
        }
    }

    impl From<super::PaletteDmc> for PaletteDmcDataIo {
        fn from(value: super::PaletteDmc) -> Self {
            let dmc_vec = value.elements
                .into_iter()
                .map(DmcDataIo::from)
                .collect();
            PaletteDmcDataIo(dmc_vec)
        }
    }

}

impl TryFrom<io::DmcDataIo> for Dmc {
    type Error = DmcError;

    fn try_from(value: io::DmcDataIo) -> Result<Self, Self::Error> {
        if value.code.is_empty() || value.color.is_empty() || value.name.is_empty() {
            return Err(Self::Error::DmcDataCorrupted);
        }

        let color = value.color;
        if !color.starts_with("#") || color.len() != 7 {
            return Err(Self::Error::HexColorParseFailed(color));
        }

        if !color[1..]
            .chars()
            .all(|c| c.is_ascii_hexdigit()) {
                return Err(Self::Error::HexColorParseFailed(color));
            }

        Ok(Dmc {
            code: value.code,
            name: value.name,
            color: palette::Srgb::new(
                u8::from_str_radix(&color[1..3], 16)?,
                u8::from_str_radix(&color[3..5], 16)?,
                u8::from_str_radix(&color[5..], 16)?,
            )
        })
    }
}

impl TryFrom<io::PaletteDmcDataIo> for PaletteDmc {
    type Error = DmcError;

    fn try_from(value: io::PaletteDmcDataIo) -> Result<Self, Self::Error> {
        // Must parse
        let dmc_vec: Result<Vec<Dmc>, Self::Error> = value.0.into_iter()
            .map(Dmc::try_from)
            .collect();
        let dmc_vec = dmc_vec?;

        // Must consist of unique names, codes and colors
        let unique_codes = dmc_vec.iter()
            .map(|dmc| &dmc.code)
            .collect::<HashSet<_>>();

        let unique_names  = dmc_vec.iter()
            .map(|dmc| &dmc.name)
            .collect::<HashSet<_>>();

        let unique_dmc = dmc_vec.iter().cloned().collect::<HashSet<_>>();

        if unique_codes.len() != unique_names.len() || unique_codes.len() != unique_dmc.len(){
            Err(Self::Error::DmcDataNotUnique)
        } else {
            Ok(Self { elements: unique_dmc})
        }
    }
}

impl PaletteDmc {
    pub fn load_from_file<P>(filepath: P) -> Result<PaletteDmc, DmcError> 
    where 
        P: AsRef<Path> + Debug
    {
        let file = std::fs::File::open(&filepath)
            .inspect_err(|_| {
                tracing::error!("File not found: '{filepath:?}'");
            })?;
        let file_reader = BufReader::new(file);
        let dmc_palette_data_io: io::PaletteDmcDataIo = serde_json::from_reader(file_reader)?;
        let dmc_palette = PaletteDmc::try_from(dmc_palette_data_io)?;
        Ok(dmc_palette)
    }
}