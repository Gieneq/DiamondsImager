use std::sync::Arc;

use axum::{
    extract::DefaultBodyLimit, 
    routing::{
        get,
        post,
        delete
    },
    Router
};

use crate::{
    app::AppData, 
    handlers::{
        overall_status, 
        upload_image,
        get_image_meta,
        delete_image,
        get_full_dmc_palette,
        start_extracting_dmc_palette,
        poll_finish_extracting_dmc_palette,
    }
};

pub fn get_router(image_size_limit: usize, app_data: Arc<AppData>) -> Router {
    let api_palette_routes = Router::new()
        .route("/dmc", get(get_full_dmc_palette)
            .with_state(app_data.clone())
        )
        .route("/extract/{uuid}", post(start_extracting_dmc_palette)
            .with_state(app_data.clone())
        )
        .route("/extract/{uuid}", get(poll_finish_extracting_dmc_palette)
            .with_state(app_data.clone())
        );

    let api_routes = Router::new()
        .route("/upload", post(upload_image)
            .layer(DefaultBodyLimit::max(image_size_limit))
            .with_state(app_data.clone())
        )
        .route("/image/{id}", get(get_image_meta)
            .with_state(app_data.clone())
        )
        .route("/image/{id}", delete(delete_image)
            .with_state(app_data.clone())
        )
        .nest("/palette", api_palette_routes);
        
    Router::new()
        .route("/", get(overall_status))
        .nest("/api", api_routes)
}

