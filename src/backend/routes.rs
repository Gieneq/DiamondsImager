use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    // Here add all domains of backend
    cfg.configure(super::image::api::routes::config);
}