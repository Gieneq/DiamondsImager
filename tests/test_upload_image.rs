use std::{error::Error, path::Path};

use diamonds_imager::results::UploadImageResult;

const TEST_IMAGES_PATH: &str = "tests/res";
const URL_ROOT: &str = "http://localhost:8080";

async fn upload_test_image(filename: &str, endpoint: &str) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();

    let filepath = Path::new(TEST_IMAGES_PATH).join(filename);
    assert!(filepath.exists());

    // let filename_no_extension = Path::new(filename).file_stem().and_then(|stem| stem.to_str()).unwrap();
    let file_extension = filepath.extension().and_then(|stem| stem.to_str()).unwrap();

    let file_bytes = tokio::fs::read(&filepath).await.expect("Cannot read file");

    let part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(filename.to_string())
        .mime_str(format!("image/{}", file_extension.to_ascii_lowercase()).as_str())?;

    let form = reqwest::multipart::Form::new().part("file", part);

    let url = format!("{URL_ROOT}{endpoint}");
    client.post(url)
        .multipart(form)
        .send()
        .await
}

#[tokio::test]
async fn test_upload_image_should_result_ok() -> Result<(), Box<dyn Error>> {
    let filename = "pinkflower_300.jpg";
    let response = upload_test_image(filename, "/upload").await;
    assert!(response.is_ok());
    let response = response.unwrap();

    if !response.status().is_success() {
        let image_upload_result_json: serde_json::Value = response.json().await?;
        panic!("Got {}", image_upload_result_json);
    } else {
        assert!(response.status().is_success(), "Upload failed: {:?}", response);
        let image_upload_result: UploadImageResult = response.json().await?;
        let filename_stem = Path::new(filename).file_stem().and_then(|stem| stem.to_str()).unwrap();
        assert!(image_upload_result.id.starts_with(filename_stem));
        assert!(image_upload_result.width > 0);
        assert!(image_upload_result.height > 0);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_upload_bigger_image_should_result_ok() -> Result<(), Box<dyn Error>> {
    let filename = "not_too_big_1_8MB.jpg";
    let response = upload_test_image(filename, "/upload").await;
    assert!(response.is_ok());
    let response = response.unwrap();

    if !response.status().is_success() {
        let image_upload_result_json: serde_json::Value = response.json().await?;
        panic!("Got {}", image_upload_result_json);
    } else {
        assert!(response.status().is_success(), "Upload failed: {:?}", response);
        let image_upload_result: UploadImageResult = response.json().await?;
        let filename_stem = Path::new(filename).file_stem().and_then(|stem| stem.to_str()).unwrap();
        assert!(image_upload_result.id.starts_with(filename_stem));
        assert!(image_upload_result.width > 0);
        assert!(image_upload_result.height > 0);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_upload_too_big_image_should_result_err() -> Result<(), Box<dyn Error>> {
    let response = upload_test_image("too_big_image_15_MB.png", "/upload").await;
    assert!(response.is_ok());
    let response = response.unwrap();
    
    assert!(!response.status().is_success());

    Ok(())
}

#[tokio::test]
async fn test_upload_too_wide_image_but_size_ok_should_result_err() -> Result<(), Box<dyn Error>> {
    let response = upload_test_image("too_wide.png", "/upload").await;
    assert!(response.is_ok());
    let response = response.unwrap();
    
    assert!(!response.status().is_success());

    Ok(())
}

#[tokio::test]
async fn test_upload_too_high_image_but_size_ok_should_result_err() -> Result<(), Box<dyn Error>> {
    let response = upload_test_image("too_high.png", "/upload").await;
    assert!(response.is_ok());
    let response = response.unwrap();
    
    assert!(!response.status().is_success());

    Ok(())
}