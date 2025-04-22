#[derive(Debug, Clone, Copy)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub address: String,
    pub port: u16,
    pub image_max_bytes: usize,
    pub image_min_size: Size<u32>,
    pub image_max_size: Size<u32>,
    pub upload_dir: String,
    pub log_level: String,
    pub workers_count: usize,
}

fn load_setting_string(key: &str) -> String {
    dotenv::var(key).unwrap_or_else(|_| panic!("Not found '{key}' in .env"))
}

fn load_setting_u16(key: &str) -> u16 {
    load_setting_string(key).parse().unwrap_or_else(|_| panic!("'{key}' value is not number"))
}

fn load_setting_u32(key: &str) -> u32 {
    load_setting_string(key).parse().unwrap_or_else(|_| panic!("'{key}' value is not number"))
}

fn load_size_u32(key_width: &str, key_height: &str) -> Size<u32> {
    Size {
        width: load_setting_u32(key_width),
        height: load_setting_u32(key_height),
    }
}

impl Settings {
    pub fn load() -> Self {
        dotenv::dotenv().ok();

        Self {
            address: load_setting_string("SERVER_ADDRESS"),
            port: load_setting_u16("SERVER_PORT"),
            image_max_bytes: (load_setting_u16("IMG_MAX_KIB") as usize) * 1024,
            image_min_size: load_size_u32("IMG_MIN_WIDTH", "IMG_MIN_HEIGHT"),
            image_max_size: load_size_u32("IMG_MAX_WIDTH", "IMG_MAX_HEIGHT"),
            upload_dir: load_setting_string("TMP_DIR_PATH"),
            log_level: dotenv::var("LOG_LEVEL").unwrap_or("info".to_string()),
            workers_count: load_setting_u16("WORKERS_COUNT") as usize,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 0,
            image_max_bytes: 4096 * 1024,
            image_min_size: Size { width: 100, height: 100 },
            image_max_size: Size { width: 5000, height: 5000 },
            upload_dir: "./tmpsf".to_string(),
            log_level: "info".to_string(),
            workers_count: 2,
        }
    }
}