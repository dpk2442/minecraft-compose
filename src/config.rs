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

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub name: String,

    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: i32,

    pub server: Server,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum Server {
    #[serde(alias = "vanilla")]
    Vanilla(VanillaServer),
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct VanillaServer {
    pub version: String,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn error::Error>> {
    log::debug!("Loading config from {}", file_path);
    let file_contents = std::fs::read_to_string(file_path)?;
    Ok(toml::from_str(&file_contents)?)
}
