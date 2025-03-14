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
        extract_palette, 
        overall_status, 
        upload_image
    }
};

pub fn get_router(image_size_limit: usize, app_data: Arc<AppData>) -> Router {
    Router::new()
        .route("/", get(overall_status))
        .route("/upload", post(upload_image)
            .layer(DefaultBodyLimit::max(image_size_limit))
            .with_state(app_data)
        )
        .route("/palette", get(extract_palette))
}