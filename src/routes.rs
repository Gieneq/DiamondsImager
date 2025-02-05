use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::scope("/api")
        .configure(crate::backend::routes::config)
    )
    .service(
        web::scope("")
        .configure(crate::frontend::routes::config)
    );
}