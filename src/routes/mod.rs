use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    use crate::handlers::api as api_handlers;
    use crate::handlers::frontend as frontend_handlers;

    cfg
    .service(
        web::scope("/api")
        .route("/", web::get().to(api_handlers::index))
        .route("/upload", web::post().to(api_handlers::image_upload))
    )
    .service(
        web::scope("")
        .route("/", web::get().to(frontend_handlers::index))
    );
}