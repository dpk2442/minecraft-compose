use serde::Deserialize;
use std::error;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    name: String,
    host: Option<String>,
    port: Option<i32>,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn error::Error>> {
    println!("Loading config from {}", file_path);
    let file_contents = std::fs::read_to_string(file_path)?;
    Ok(toml::from_str(&file_contents)?)
}
