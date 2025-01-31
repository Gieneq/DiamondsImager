use actix_web::web;

mod frontend {
    use actix_files::NamedFile;

    pub async fn index() -> actix_web::Result<NamedFile> {
        log::info!("frontend_index");
        let file_content = NamedFile::open("static/index.html").unwrap();
        Ok(file_content)
    }
}

mod api {
    use actix_multipart::form::{tempfile::TempFile, MultipartForm};
    use actix_web::{HttpResponse, Responder};
    
    use crate::handlers;

    pub async fn index() -> impl Responder {
        log::info!("index");
        let body_content = handlers::index();
        HttpResponse::Ok().body(body_content)
    }

    // #[derive(Debug, Deserialize)]
    // struct Metadata {
    //     name: String,
    // }

    // #[derive(Debug, MultipartForm)]
    // struct UploadForm {
    //     #[multipart(limit = "100MB")]
    //     file: TempFile,
    //     // json: MPJson<Metadata>,
    // }

    // async fn image_upload(form: actix_form_data::Value<()>) -> impl Responder {
    //     log::info!("image_upload form={form:?}");
    //     let _body_content = handlers::image_upload();
    //     HttpResponse::Ok().body("Hi from app")
    // }

    // curl -v --request POST --url http://localhost:8080/api/upload -F 'json={"name": "Cargo.lock"};type=application/json' -F file=@./Cargo.lock

    // async fn image_upload(MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
    //     log::info!("image_upload form={form:?}");
    //     let _body_content = handlers::image_upload();
    //     HttpResponse::Ok().body("Hi from app")
    // }

    #[derive(Debug, MultipartForm)]
    pub struct UploadForm {
        #[multipart(rename = "file")]
        file: TempFile,
    }

    pub async fn image_upload(MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
        log::info!("image_upload form={form:?}");
        // payload.
        // let _body_content = handlers::image_upload();

        // while let Ok(Some(mut field)) = payload.try_next().await {
        //     let content_type = field.content_disposition().unwrap();
        //     //let filename = content_type.get_filename().unwrap();
        // }
        let f = form.file ;
        
        {
            let path = format!("./tmp/{}", f.file_name.unwrap());
            log::info!("saving to {path}");
            f.file.persist(path).unwrap();
        }

        // Ok(HttpResponse::Ok())
        HttpResponse::Ok().body("Hi from app")
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::scope("")
        .route("/", web::get().to(frontend::index))
        .service(
            web::scope("/api")
            .route("/", web::get().to(api::index))
            .route("/upload", web::post().to(api::image_upload))
        )
    );
}