use std::path::Path;

pub mod settings;
pub mod router;
pub mod handlers;
pub mod requests;
pub mod results;
pub mod errors;
pub mod services;
pub mod app;

pub async fn recreate_dir<P: AsRef<Path>>(dir: P) -> tokio::io::Result<()> {
    let dirpath = dir.as_ref();
    
    if dirpath.exists() {
        tokio::fs::remove_dir_all(&dirpath).await.unwrap_or_else(|e| {
            panic!("Failed to remove content of '{:?}', reason: {}", dirpath, e)
        });
    }

    tokio::fs::create_dir_all(dirpath).await.unwrap_or_else(|e| {
        panic!("Failed to create results directory, reason: {}", e)
    });

    Ok(())
}