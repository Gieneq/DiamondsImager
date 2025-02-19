use diamonds_imager::{routes, settings::{Settings, system}, tools};

use actix_web::{
    middleware::Logger, web, App, HttpServer
};

fn prepare() -> std::io::Result<Settings> {
    let settings = Settings::load();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or(&settings.log_level));

    log::debug!("Loaded settings: {settings:?}");
    
    system::setup(&settings);

    tools::clear_files_in_dir(&settings.tmp_path)?;

    Ok(settings)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = prepare()?;

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

