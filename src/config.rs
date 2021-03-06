use serde::Deserialize;
use std::collections::HashMap;
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

    default_world_name -> String: "world".to_string();
    default_world_gamemode -> String: "survival".to_string();
    default_world_allow_flight -> bool: false;
    default_world -> World: World {
        name: default_world_name(),
        seed: None,
        gamemode: default_world_gamemode(),
        allow_flight: default_world_allow_flight(),
    };

}

#[derive(Clone, Deserialize, Debug, Default, PartialEq)]
pub struct Config {
    pub name: String,

    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: i32,

    pub server: Server,

    #[serde(default = "default_world")]
    pub world: World,

    pub datapacks: Option<HashMap<String, String>>,
}

#[derive(Clone, Deserialize, Debug, Default, PartialEq)]
pub struct Server {
    pub version: String,

    pub memory: Option<String>,

    #[serde(flatten)]
    pub server_type: ServerType,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ServerType {
    #[serde(alias = "vanilla")]
    Vanilla,
}

impl Default for ServerType {
    fn default() -> ServerType {
        ServerType::Vanilla
    }
}

#[derive(Clone, Deserialize, Debug, Default, PartialEq)]
pub struct World {
    #[serde(default = "default_world_name")]
    pub name: String,

    pub seed: Option<String>,

    #[serde(default = "default_world_gamemode")]
    pub gamemode: String,

    #[serde(default = "default_world_allow_flight")]
    pub allow_flight: bool,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn error::Error>> {
    log::debug!("Loading config from {}", file_path);
    let file_contents = std::fs::read_to_string(file_path)?;
    Ok(toml::from_str(&file_contents)?)
}
