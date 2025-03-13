pub mod settings;
pub mod router;
pub mod handlers;

use crate::settings::Settings;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() {
    let settings = Settings::load();
    
    // Set tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = router::get_router();

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", settings.address, settings.port))
        .await
        .expect("Could not start listener");
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    
    axum::serve(listener, app)
        .await
        .unwrap();
}