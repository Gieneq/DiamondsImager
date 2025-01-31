#[derive(Debug)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

#[derive(Debug)]
pub struct Settings {
    pub address: String,
    pub port: u16,
    pub image_min_size: Size<u16>,
    pub image_max_size: Size<u16>,
}

fn load_setting_string(key: &str) -> String {
    dotenv::var(key).unwrap_or_else(|_| panic!("Not found '{key}' in .env"))
}

fn load_setting_u16(key: &str) -> u16 {
    load_setting_string(key).parse().unwrap_or_else(|_| panic!("'{key}' value is not number"))
}

fn load_size_u16(key_width: &str, key_height: &str) -> Size<u16> {
    Size {
        width: load_setting_u16(key_width),
        height: load_setting_u16(key_height),
    }
}

impl Settings {
    pub fn load() -> Self {
        dotenv::dotenv().ok();

        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

        let result = Self {
            address: load_setting_string("SERVER_ADDRESS"),
            port: load_setting_u16("SERVER_PORT"),
            image_min_size: load_size_u16("IMG_MIN_WIDTH", "IMG_MIN_HEIGHT"),
            image_max_size: load_size_u16("IMG_MAX_WIDTH", "IMG_MAX_HEIGHT"),
        };
        
        log::info!("Loaded settings: {result:?}");

        result
    }
}