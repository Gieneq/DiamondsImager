use std::{
    path::Path,
    future::Future
};

use diamonds_imager::app::app_serve;
use diamonds_imager::results::{GetPaletteResult, UploadImageResult};
use diamonds_imager::services::{ImageId, ImageStorageMeta};
use diamonds_imager::settings::Settings;
use reqwest::Client;

const TEST_IMAGES_PATH: &str = "tests/res";

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer() // Output for tests
        .try_init();
}

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

async fn get_test_full_dmc_palette(root_url: &str, client: &Client) -> Result<GetPaletteResult, reqwest::Error> {
    let response = client.get(format!("{root_url}/api/palette/dmc"))
        .send()
        .await?;
    response.json().await
}

async fn upload_basic_good_image(root_url: &str, client: &Client) -> Result<UploadImageResult, reqwest::Error> {
    let filename = "pinkflower_300.jpg"; // we call this image coala, wy wife says it looks like coala. lol
    let response = upload_test_image(root_url, client, filename, "/api/upload").await?;
    assert!(response.status().is_success());
    response.json().await
}

async fn get_test_image_meta(root_url: &str, client: &Client, id: &ImageId) -> Result<Option<ImageStorageMeta>, reqwest::Error> {
    let response = client.get(format!("{root_url}/api/image/{id}"))
    .send()
    .await?;

    Ok(if response.status().is_success() {
        Some(response.json().await?)
    } else {
        None
    })    
}

async fn delete_test_image(root_url: &str, client: &Client, id: &ImageId) -> Result<bool, reqwest::Error> {
    let response = client.delete(format!("{root_url}/api/image/{id}"))
    .send()
    .await?;

    Ok(response.status().is_success())
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
        init_tracing();

        setup_server_environment_with_client( |root_url, client| async move {
            let response = upload_test_image(
                &root_url, 
                &client, 
                "too_big_image_15_MB.png", 
                "/api/upload"
            ).await;

            // Weird response, behaves differently depending on OS
            #[cfg(target_os = "linux")]
            {
                let response = response.unwrap(); // Linux: unwrap Result<reqwest::Response>
                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            }

            #[cfg(target_os = "windows")]
            {
                assert!(response.is_err(), "Expected connection error on Windows");
            }

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
mod test_palette {
    use crate::*;

    #[tokio::test]
    async fn test_get_full_dmc_palette() {
        setup_server_environment_with_client( |root_url, client| async move {
            let response_palette_dmc = get_test_full_dmc_palette(&root_url, &client).await.unwrap();
            
            assert!(!response_palette_dmc.palette.as_ref().is_empty());
        }).await;
    }
}

#[cfg(test)]
mod test_image_access_delete {
    use crate::*;

    #[tokio::test]
    async fn test_get_image_meta() {
        setup_server_environment_with_client( |root_url, client| async move {
            let upload_result = upload_basic_good_image(&root_url, &client).await.unwrap();
            
            let id = upload_result.id;
            assert!(!id.is_empty());

            // Get image meta
            let image_meta_should_be_ok = get_test_image_meta(&root_url, &client, &id).await.unwrap().unwrap();
            println!("{image_meta_should_be_ok:?}");

            let was_deleted = delete_test_image(&root_url, &client, &id).await.unwrap();
            assert!(was_deleted);

            let was_deleted = delete_test_image(&root_url, &client, &id).await.unwrap();
            assert!(!was_deleted);

            let image_meta_result_should_be_none = get_test_image_meta(&root_url, &client, &id).await.unwrap();
            assert!(image_meta_result_should_be_none.is_none());
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