use std::{
    net::SocketAddr, 
    path::PathBuf, 
    sync::Arc
};

use tokio::sync::Mutex;
use tower_http::trace::{
    DefaultMakeSpan, 
    DefaultOnRequest, 
    DefaultOnResponse, 
    TraceLayer
};

use crate::{
    router, 
    services::{
        dmc::PaletteDmc, processing::WorkDispatcher, ImageStorageService
    }, 
    settings::Settings
};

#[derive(Debug)]
pub struct AppData {
    pub image_max_width: u32,
    pub image_max_height: u32,
    pub palette_dmc_full: Arc<PaletteDmc>,
    pub image_storage_service: tokio::sync::Mutex<ImageStorageService>,
    pub processing_runner_service: tokio::sync::Mutex<WorkDispatcher>,
}

impl Default for AppData {
    fn default() -> Self {
        Self { 
            image_max_width: 1024, 
            image_max_height: 1024, 
            palette_dmc_full: Arc::new(PaletteDmc::default()),
            image_storage_service: Mutex::new(ImageStorageService::new()),
            processing_runner_service: Mutex::new(WorkDispatcher::new()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppServeError {
    #[error("IoError reson='{0}'")]
    IoError(#[from] tokio::io::Error),
}

#[derive(Debug)]
pub struct AppServeHandler {
    task_handle: tokio::task::JoinHandle<Result<(), std::io::Error>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    pub address: SocketAddr
}

impl AppServeHandler {
    pub fn shutdown_gracefully(&mut self) {
        tracing::info!("Gracefully shutting down...");
        self.shutdown_tx
            .take()
            .expect("Shutdown send was already used")
            .send(())
            .expect("Should send");
    }
    
    pub async fn await_shutdown(self) -> Result<(), std::io::Error> {
        self.task_handle.await?
    }
    
    pub async fn shutdown_gracefully_await(mut self) -> Result<(), std::io::Error> {
        self.shutdown_gracefully();
        self.await_shutdown().await
    }
    
    pub fn get_url(&self) -> String {
        format!("http://{}", self.address)
    }
}

pub async fn app_serve(settings: Settings) -> Result<AppServeHandler, AppServeError> {
    let dmc_palette_filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&settings.dmc_palette_path);

    let palette_dmc_full = PaletteDmc::load_from_file(dmc_palette_filepath).expect("DMC Fullpalette file should exist");
    let app_data = Arc::new(AppData {
        image_max_width: settings.image_max_size.width,
        image_max_height: settings.image_max_size.height,
        palette_dmc_full: Arc::new(palette_dmc_full),
        ..Default::default()
    });

    let app = router::get_router(settings.image_max_bytes, app_data.clone())
        .layer(TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(tracing::Level::DEBUG))
            .on_response(DefaultOnResponse::new().level(tracing::Level::DEBUG)),
        );

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", settings.address, settings.port)).await?;
    let address = listener.local_addr()?;

    tracing::info!("{}({}) listening on {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), address);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let task_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_rx.await.ok();
            })
            .await
    });
    
    Ok(AppServeHandler {
        task_handle,
        shutdown_tx: Some(shutdown_tx),
        address,
    })
}
