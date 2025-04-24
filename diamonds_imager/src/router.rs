use std::sync::Arc;

use axum::{
    extract::DefaultBodyLimit, 
    routing::{
        get,
        post
    },
    Router
};

use crate::{
    app::AppData, 
    handlers::{
        overall_status, 
        upload_image,
    }
};

pub fn get_router(image_size_limit: usize, app_data: Arc<AppData>) -> Router {
    let api_routes = Router::new()
    .route("/upload", post(upload_image)
        .layer(DefaultBodyLimit::max(image_size_limit))
        .with_state(app_data.clone())
    );

    Router::new()
        .route("/", get(overall_status))
        .nest("/api", api_routes)
}