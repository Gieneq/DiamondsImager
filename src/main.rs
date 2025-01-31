use diamonds_imager::{routes, settings::Settings};

use actix_web::{
    middleware::Logger,
    App, HttpServer,
};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::load();

    log::info!("starting HTTP server at http://{}:{}", settings.address, settings.port);

    HttpServer::new(move || {
        App::new()
            // .app_data(tmpl_reloader.clone())
            .configure(routes::config)
            .wrap(Logger::default())
    })
    .workers(2)
    .bind((settings.address.as_str(), settings.port))?
    .run()
    .await
}

