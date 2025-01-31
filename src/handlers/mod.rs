// use serde::Deserialize;

use crate::services;

// #[derive(Debug, Deserialize)]
// struct Info {
//     name: String,
// }

pub fn index() -> String {
    services::get_hello_message()
}

pub fn image_upload() -> () {
    services::process_image();
}
