use actix_files::NamedFile;

pub async fn index() -> actix_web::Result<NamedFile> {
    log::info!("frontend_index");
    let file_content = NamedFile::open("static/index.html").unwrap();
    Ok(file_content)
}