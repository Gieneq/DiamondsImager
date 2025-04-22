use std::{
    net::SocketAddr, path::PathBuf, sync::Arc
};

use diamonds_imager_generator::dmc::PaletteDmc;

use tower_http::trace::{
    DefaultMakeSpan, 
    DefaultOnRequest, 
    DefaultOnResponse, 
    TraceLayer
};

use crate::{
    recreate_dir, router, settings::Settings
};

#[derive(Debug, Clone)]
pub struct AppData {
    pub uplad_dir: PathBuf,
    pub image_max_width: usize,
    pub image_max_height: usize,
    pub dmc_full_palette: PaletteDmc,
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
    let upload_dir_path = PathBuf::from(&settings.upload_dir);
    
    recreate_dir(&upload_dir_path).await?;

    let dmc_full_palette = PaletteDmc::load_dmc_palette().unwrap();
    let app_data = Arc::new(AppData {
        uplad_dir: upload_dir_path,
        image_max_width: settings.image_max_size.width as usize,
        image_max_height: settings.image_max_size.height as usize,
        dmc_full_palette
    });

    let app = router::get_router(settings.image_max_bytes, app_data.clone())
        .layer(TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(tracing::Level::DEBUG))
            .on_response(DefaultOnResponse::new().level(tracing::Level::DEBUG)),
        );

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", settings.address, settings.port)).await?;
    let address = listener.local_addr()?;

    tracing::info!("listening on {}", address);

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
