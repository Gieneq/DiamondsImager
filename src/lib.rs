pub mod routes;
pub mod settings;
pub mod handlers;
pub mod services;

pub mod tools {
    use std::{io, path::Path, fs};

    pub fn clear_files_in_dir<P: AsRef<Path>>(dir: P) -> io::Result<()> {
        let dirpath = dir.as_ref();
        
        match fs::remove_dir_all(dirpath) {
            Ok(_) => { log::info!("Dir {dirpath:?} got removed with its content") },
            Err(e) => { log::warn!("Dir {dirpath:?} could not be removed, reson = {e}") },
        }

        fs::create_dir(dirpath).map(|_| {
            log::debug!("Dir {dirpath:?} recreated")
        })
    }
}