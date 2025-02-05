pub mod frontend {
    use actix_files::NamedFile;

    pub async fn index() -> actix_web::Result<NamedFile> {
        log::info!("frontend_index");
        let file_content = NamedFile::open("static/index.html").unwrap();
        Ok(file_content)
    }
}

pub mod api {
    use actix_multipart::form::{tempfile::TempFile, MultipartForm};
    use actix_web::{http::StatusCode, web, HttpResponse, Responder, ResponseError};
    use serde::Serialize;

    use crate::{
        services::processing::{self as processing_service, InputFileError, ProcessingFileError}, 
        settings::Settings
    };

    #[derive(Debug, Serialize)]
    struct ErrorResponse {
        error: String,
    }

    #[derive(Debug, MultipartForm)]
    pub struct UploadForm {
        #[multipart(rename = "file")]
        file: TempFile,
    }

    #[derive(Debug, Serialize)]
    struct ImageUploadResult {
        new_filename: String,
    }

    impl ResponseError for InputFileError {
        fn status_code(&self) -> actix_web::http::StatusCode {
            StatusCode::BAD_REQUEST
        }

        fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
            HttpResponse::build(self.status_code()).json(ErrorResponse {
                error: self.to_string()
            })
        }
    }

    impl From<InputFileError> for Box<dyn ResponseError> {
        fn from(value: InputFileError) -> Self {
            Box::new(value)
        }
    }

    impl ResponseError for ProcessingFileError {
        fn status_code(&self) -> actix_web::http::StatusCode {
            StatusCode::BAD_REQUEST
        }

        fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
            HttpResponse::build(self.status_code()).json(ErrorResponse {
                error: self.to_string()
            })
        }
    }

    impl From<ProcessingFileError> for Box<dyn ResponseError> {
        fn from(value: ProcessingFileError) -> Self {
            Box::new(value)
        }
    }

    pub async fn index() -> impl Responder {
        log::info!("entered");
        HttpResponse::Ok().body("API index")
    }

    pub async fn image_upload(
        settings_appdata: web::Data<Settings>,
        MultipartForm(form): MultipartForm<UploadForm>
    ) -> Result<HttpResponse, Box<dyn ResponseError>> {
        log::info!("form={form:?}, settings_appdata={settings_appdata:?}");

        let file = validate_input_file(form.file)?;

        let result_filename = processing_service::save_file_with_generated_unique_filename(&settings_appdata, file)?;

        Ok(HttpResponse::Ok().json(ImageUploadResult { new_filename: result_filename }))
    }

    fn validate_input_file(file: TempFile) -> Result<TempFile, InputFileError> {
        if file.size == 0 {
            Err(InputFileError::FileEmpty)
        } else if file.file_name.is_none() {
            Err(InputFileError::FileNoname)
        } else {
            Ok(file)
        }
    }
}


    // pub async fn generate_preview_image() -> impl Responder {

    // }

    // pub async fn generate_final_pdf() -> impl Responder {
        
    // }
    
        // ////////

        // let processing_result = processing_service::process_image(
        //     unique_filepath,
        //     settings_appdata.image_min_size,
        //     settings_appdata.image_max_size,
        //     &Constrins::default()
        // );

        // let processed_image = match processing_result {
        //     Ok(file) => file,
        //     Err(e) => { return e.error_response(); },
        // };

        // let response = format!("Processed iamge saved here: '{processed_image}'");
        // HttpResponse::Ok().body(response)