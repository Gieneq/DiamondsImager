use std::sync::Arc;

use diamonds_imager::{
    app::app_serve, 
    settings::Settings
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let settings = Settings::load();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(settings.log_level.clone()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut serve_handler = app_serve(settings).await.unwrap();
    
    let ctrlc_notify = Arc::new(tokio::sync::Notify::new());
    let ctrlc_notify_cloned = ctrlc_notify.clone();

    ctrlc::set_handler(move || {
        ctrlc_notify_cloned.notify_one();
    }).expect("Error setting Ctrl-C handler");

    // Wait until Ctrl+C is triggered
    ctrlc_notify.notified().await;

    serve_handler.shutdown_gracefully();
    if let Err(e) = serve_handler.await_shutdown().await {
        eprintln!("Server shutdown error: {}", e);
    }

    println!("Shutdown complete.");
}