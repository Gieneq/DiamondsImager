use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::scope("/image")
        .route("", web::get().to(super::handlers::index))
        .route("/new", web::post().to(super::handlers::image_upload))
    );
}