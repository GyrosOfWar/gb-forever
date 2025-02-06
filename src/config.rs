use camino::Utf8PathBuf;
use config::{Config, Environment, File};
use serde::Deserialize;

use crate::Result;

#[derive(Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub stream_key: String,
    pub video_path: Utf8PathBuf,
}

pub fn load_config() -> Result<AppConfig> {
    Config::builder()
        .add_source(File::with_name("gb-forever").required(false))
        .add_source(Environment::default())
        .build()?
        .try_deserialize()
        .map_err(From::from)
}
