use diamonds_imager::{routes, settings::Settings, tools};

use actix_web::{
    middleware::Logger, web, App, HttpServer
};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::load();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or(&settings.log_level));
        
    log::debug!("Loaded settings: {settings:?}");

    tools::clear_files_in_dir(&settings.tmp_path)?;

    log::info!("Starting HTTP server at http://{}:{}", settings.address, settings.port);

    let settings_appdata = web::Data::new(settings.clone());
    HttpServer::new(move || {
        App::new()
            .app_data(settings_appdata.clone())
            .configure(routes::config)
            .wrap(Logger::default())
    })
    .workers(settings.workers_count)
    .bind((settings.address.as_str(), settings.port))?
    .run()
    .await
}

