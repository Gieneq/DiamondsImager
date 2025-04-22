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
        dmc_full_palette, 
        overall_status, 
        upload_image,
        processings_status
    }
};

pub fn get_router(image_size_limit: usize, app_data: Arc<AppData>) -> Router {
    Router::new()
        .route("/", get(overall_status))
        .route("/upload", post(upload_image)
            .layer(DefaultBodyLimit::max(image_size_limit))
            .with_state(app_data.clone())
        )
        .route("/palette/dmc", get(dmc_full_palette)
            .with_state(app_data.clone())
        )
        .route("/processings", get(processings_status)
            .with_state(app_data.clone())
        )
}