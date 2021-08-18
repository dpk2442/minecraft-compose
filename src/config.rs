use serde::Deserialize;
use std::error;
use toml;

macro_rules! config_defaults {
    ($($name:ident -> $type:ty: $value:expr;)*) => {
    $(
        fn $name() -> $type {
            $value
        }
    )*
    }
}

config_defaults! {
    default_host -> String: "0.0.0.0".to_string();
    default_port -> i32: 25565;
}

#[derive(Deserialize, Debug)]
pub struct Config {
    name: String,

    #[serde(default = "default_host")]
    host: String,

    #[serde(default = "default_port")]
    port: i32,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn error::Error>> {
    log::debug!("Loading config from {}", file_path);
    let file_contents = std::fs::read_to_string(file_path)?;
    Ok(toml::from_str(&file_contents)?)
}
