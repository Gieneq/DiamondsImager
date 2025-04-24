use std::{
    path::Path,
    future::Future
};

use diamonds_imager::app::app_serve;
use diamonds_imager::results::UploadImageResult;
use diamonds_imager::settings::Settings;
use reqwest::Client;

const TEST_IMAGES_PATH: &str = "tests/res";

async fn upload_test_image(root_url: &str, client: &Client, filename: &str, endpoint: &str) -> Result<reqwest::Response, reqwest::Error> {
    let filepath = Path::new(TEST_IMAGES_PATH).join(filename);
    assert!(filepath.exists());

    let file_extension = filepath.extension().and_then(|stem| stem.to_str()).unwrap();

    let file_bytes = tokio::fs::read(&filepath).await.expect("Cannot read file");

    let part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(filename.to_string())
        .mime_str(format!("image/{}", file_extension.to_ascii_lowercase()).as_str())?;

    let form = reqwest::multipart::Form::new().part("file", part);

    client.post(format!("{root_url}{endpoint}"))
        .multipart(form)
        .send()
        .await
}

async fn upload_basic_good_image(root_url: &str, client: &Client) -> Result<UploadImageResult, reqwest::Error> {
    let filename = "pinkflower_300.jpg"; // we call this image coala, wy wife says it looks like coala. lol
    let response = upload_test_image(root_url, client, filename, "/api/upload").await?;
    assert!(response.status().is_success());
    response.json().await
}

static SERVER_LOCK: tokio::sync::OnceCell<tokio::sync::Mutex<()>> = tokio::sync::OnceCell::const_new();

async fn acquire_server_lock<'a>() -> tokio::sync::MutexGuard<'a, ()> {
    let lock = SERVER_LOCK.get_or_init(|| async {
        tokio::sync::Mutex::new(())
    }).await;
    lock.lock().await
}

async fn setup_server_environment_with_client<F, Fut>(test_procedure: F)
where 
    F: FnOnce(String, Client) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static
{
    let _guard = acquire_server_lock().await;
    {
        let serve_handle = app_serve(Settings::default()).await.unwrap();
        let client = reqwest::Client::new();
    
        test_procedure(serve_handle.get_url(), client).await;
    
        serve_handle.shutdown_gracefully_await().await.unwrap();
    }
}

#[cfg(test)]
mod test_status {
    use super::*;

    #[tokio::test]
    async fn test_check_status() {
        setup_server_environment_with_client( |root_url, client| async move {
            let response = client.get(root_url + "/")
            .send()
            .await.unwrap();
    
            let response_text = response.text().await.unwrap();
            assert_eq!(response_text, "<h1>Diamonds imager is running!</h1>".to_string());
        }).await;
    }
}

#[cfg(test)]
mod test_uploading_image {
    use super::*;

    #[tokio::test]
    async fn test_upload_image_should_result_ok() {
        setup_server_environment_with_client( |root_url, client| async move {
            let filename = "pinkflower_300.jpg";

            let response = upload_test_image(
                &root_url, 
                &client, 
                filename, 
                "/api/upload"
            ).await.unwrap();

            if !response.status().is_success() {
                let image_upload_result_json: serde_json::Value = response.json().await.unwrap();
                panic!("Got {}", image_upload_result_json);
            } else {
                assert!(response.status().is_success(), "Upload failed: {:?}", response);
                let image_upload_result: UploadImageResult = response.json().await.unwrap();
                let filename_stem = Path::new(filename).file_stem().and_then(|stem| stem.to_str()).unwrap();
                assert!(image_upload_result.id.starts_with(filename_stem));
                assert!(image_upload_result.width > 0);
                assert!(image_upload_result.height > 0);
            }
        }).await;
    }

    #[tokio::test]
    async fn test_upload_bigger_image_should_result_ok() {
        setup_server_environment_with_client( |root_url, client| async move {
            let filename = "not_too_big_1_8MB.jpg";

            let response = upload_test_image(
                &root_url, 
                &client, 
                filename, 
                "/api/upload"
            ).await.unwrap();

            if !response.status().is_success() {
                let image_upload_result_json: serde_json::Value = response.json().await.unwrap();
                panic!("Got {}", image_upload_result_json);
            } else {
                assert!(response.status().is_success(), "Upload failed: {:?}", response);
                let image_upload_result: UploadImageResult = response.json().await.unwrap();
                let filename_stem = Path::new(filename).file_stem().and_then(|stem| stem.to_str()).unwrap();
                assert!(image_upload_result.id.starts_with(filename_stem));
                assert!(image_upload_result.width > 0);
                assert!(image_upload_result.height > 0);
            }
        }).await;
    }

    #[tokio::test]
    async fn test_upload_too_big_image_should_result_err() {
        setup_server_environment_with_client( |root_url, client| async move {
            let response = upload_test_image(
                &root_url, 
                &client, 
                "too_big_image_15_MB.png", 
                "/api/upload"
            ).await;
            assert!(response.is_err());
        }).await;
    }

    #[tokio::test]
    async fn test_upload_too_wide_image_but_size_ok_should_result_err() {
        setup_server_environment_with_client( |root_url, client| async move {
            let response = upload_test_image(
                &root_url, 
                &client, 
                "too_wide.png", 
                "/api/upload"
            ).await.unwrap();
            
            assert!(!response.status().is_success());
        }).await;
    }

    #[tokio::test]
    async fn test_upload_too_high_image_but_size_ok_should_result_err() {
        setup_server_environment_with_client( |root_url, client| async move {
            let response = upload_test_image(
                &root_url, 
                &client, 
                "too_high.png", 
                "/api/upload"
            ).await.unwrap();
            
            assert!(!response.status().is_success());
        }).await;
    }

    #[tokio::test]
    async fn test_upload_multiple_identical_image_different_ids() {
        setup_server_environment_with_client( |root_url, client| async move {
            let upload_img_result1 = upload_basic_good_image(&root_url, &client).await.unwrap();
            let upload_img_result2 = upload_basic_good_image(&root_url, &client).await.unwrap();
            assert_ne!(upload_img_result1.id, upload_img_result2.id);
            assert_eq!(upload_img_result1.width, upload_img_result2.width);
            assert_eq!(upload_img_result1.height, upload_img_result2.height);
        }).await;
    }
}

#[cfg(test)]
mod test_processing {
    use super::*;

    // #[tokio::test]
    // async fn upload_good_image_check_status_should_be_ready_for_processing() {
    //     setup_server_environment_with_client( |root_url, client| async move {
    //         let upload_img_result = upload_basic_good_image(&root_url, &client).await.unwrap();
    //         println!("Got upload_img_result={upload_img_result:?}");

    //         // Check status endpoint like this /processing/{id}/status
    //         //TODO
    //         }).await;
    // }
}