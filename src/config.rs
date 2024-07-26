use std::fs;

use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GlobalConfig {
    pub personal_access_token: Option<String>,
}

impl GlobalConfig {
    pub fn read() -> Result<GlobalConfig> {
        let os_home = std::env::var("HOME")?;
        if let Ok(file_content) = fs::read_to_string(format!("{os_home}/.gstack/config.toml")) {
            let config: GlobalConfig = toml::from_str(file_content.as_str())?;
            Ok(config)
        } else {
            bail!("Could not find gstack config at default location $HOME/.gstack/config.toml")
        }
    }
}
