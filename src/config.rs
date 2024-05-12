use std::fs;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GlobalConfig {
    pub personal_access_token: Option<String>,
}

impl GlobalConfig {
    pub fn read() -> Result<GlobalConfig> {
        let os_home = std::env::var("HOME")?;
        let file_content = fs::read_to_string(format!("{os_home}/.gstack/config.toml"))?;
        let config: GlobalConfig = toml::from_str(file_content.as_str())?;
        Ok(config)
    }
}
