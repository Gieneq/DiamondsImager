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

async fn upload_basic_good_image() -> Result<UploadImageResult, reqwest::Error> {
    let filename = "pinkflower_300.jpg"; // we call this image coala, wy wife says it looks like coala. lol
    let response = upload_test_image(filename, "/upload").await?;
    assert!(response.status().is_success());
    response.json().await
}

#[cfg(test)]
mod test_status {
    use super::*;

    #[tokio::test]
    async fn test_check_status() -> Result<(), Box<dyn Error>> {
        let client = reqwest::Client::new();

        let url = format!("{URL_ROOT}/");

        let response = client.get(url)
        .send()
        .await?;

        let response_text = response.text().await?;
        assert_eq!(response_text, "<h1>Diamonds imager is running!</h1>".to_string());

        Ok(())
    }
}

#[cfg(test)]
mod test_uploading_image {
    use super::*;

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

    #[tokio::test]
    async fn test_upload_multiple_identical_image_different_ids() -> Result<(), Box<dyn Error>> {
        let upload_img_result1 = upload_basic_good_image().await;
        let upload_img_result2 = upload_basic_good_image().await;
        assert!(upload_img_result1.is_ok(), "Uploading image failed, reason {upload_img_result1:?}");
        assert!(upload_img_result2.is_ok(), "Uploading image failed, reason {upload_img_result2:?}");
        let upload_img_result1 = upload_img_result1.unwrap();
        let upload_img_result2 = upload_img_result2.unwrap();
        assert_ne!(upload_img_result1.id, upload_img_result2.id);
        assert_eq!(upload_img_result1.width, upload_img_result2.width);
        assert_eq!(upload_img_result1.height, upload_img_result2.height);
        Ok(())
    }
}

#[cfg(test)]
mod test_processing {
    use super::*;

    #[tokio::test]
    async fn upload_good_image_check_status_should_be_ready_for_processing() {
        let upload_img_result = upload_basic_good_image().await;
        assert!(upload_img_result.is_ok(), "Uploading image failed, reason {upload_img_result:?}");
        let upload_img_result = upload_img_result.unwrap();
        println!("Got upload_img_result={upload_img_result:?}");

        // Check status endpoint like this /processing/{id}/status
        //TODO
    }
}