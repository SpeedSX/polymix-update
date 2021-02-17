use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct UpdateMode {
    pub name: String,
    pub file_mask: String,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub connection_string: String,
    pub update_mode: Vec<UpdateMode>,
}

pub fn get_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let file = File::open(path.as_ref()).with_context(|| format!("Failed to open config file {}", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}
