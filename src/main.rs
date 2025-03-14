pub mod settings;
pub mod router;
pub mod handlers;
pub mod requests;
pub mod results;
pub mod errors;
pub mod services;
pub mod app;

use std::{
    path::{
        Path, 
        PathBuf
    }, 
    sync::Arc
};

use tracing_subscriber::{
    layer::SubscriberExt, 
    util::SubscriberInitExt
};
use tower_http::trace::{
    TraceLayer, 
    DefaultMakeSpan, 
    DefaultOnRequest, 
    DefaultOnResponse
};

use crate::settings::Settings;
use crate::app::AppData;

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

#[tokio::main]
async fn main() {
    let settings = Settings::load();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(settings.log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let upload_dir_path = PathBuf::from(&settings.upload_dir);
    recreate_dir(&upload_dir_path)
        .await
        .expect("could not recreate upload dir");

    let app_data = Arc::new(AppData {
        uplad_dir: upload_dir_path,
        image_max_width: settings.image_max_size.width as usize,
        image_max_height: settings.image_max_size.height as usize
    });

    let app = router::get_router(settings.image_max_bytes, app_data.clone())
        .layer(TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(tracing::Level::DEBUG))
            .on_response(DefaultOnResponse::new().level(tracing::Level::DEBUG)),
        );

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", settings.address, settings.port))
        .await
        .expect("Could not start listener");
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    
    axum::serve(listener, app)
        .await
        .unwrap();
}