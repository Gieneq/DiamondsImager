use diamonds_imager::backend::image::api::responses::ImageUploadResult;
use reqwest::multipart;
use std::error::Error;

#[tokio::test]
async fn test_upload_image() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();

    let filename_no_extension = "test_ok_image";
    let file_extension = "jpg";
    let filename = format!("{filename_no_extension}.{file_extension}");
    let image_path = format!("tests/res/{filename}");
    let file_bytes = tokio::fs::read(&image_path).await?;

    let part = multipart::Part::bytes(file_bytes)
        .file_name(filename)
        .mime_str("image/jpeg")?;

    let form = multipart::Form::new().part("file", part);

    let endpoint = "/api/image/new";
    let url = format!("http://localhost:8080{endpoint}");
    let response = client.post(url)
        .multipart(form)
        .send()
        .await?;

    assert!(response.status().is_success(), "Upload failed: {:?}", response.status());
    let response_text: ImageUploadResult = response.json().await?;
    assert!(response_text.saved_filename.starts_with(filename_no_extension));
    assert!(response_text.saved_filename.ends_with(file_extension));
    
    Ok(())
}
