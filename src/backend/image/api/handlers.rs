use actix_multipart::form::MultipartForm;
use actix_web::{
    web, HttpResponse, Responder, ResponseError
};

use crate::settings::Settings;

use super::{
    forms::UploadForm,
    responses::ImageUploadResult,
    super::services
};

pub async fn index() -> impl Responder {
    log::info!("entered");
    HttpResponse::Ok().body("API index")
}

pub async fn image_upload(
    settings_appdata: web::Data<Settings>,
    MultipartForm(form): MultipartForm<UploadForm>
) -> Result<HttpResponse, Box<dyn ResponseError>> {
    log::info!("form={form:?}, settings_appdata={settings_appdata:?}");
    
    let result_filename = services::save_file_with_generated_unique_filename(&settings_appdata, form.file)?;

    Ok(HttpResponse::Ok().json(ImageUploadResult { new_filename: result_filename }))
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