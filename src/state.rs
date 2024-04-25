use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GitStack {
    pub prefix: Option<String>,
    pub base_branch: String,
    pub branches: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GsState {
    pub stacks: Vec<GitStack>,
}

impl GsState {
    pub fn init(base_path: PathBuf) -> Result<GsState> {
        let state = match File::open(base_path.clone().join(".git/gstack/state.ron")) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                ron::from_str(&contents)?
            }
            Err(_) => {
                fs::create_dir(base_path.clone().join(".git/gstack"));
                GsState::default()
            }
        };
        Ok(state)
    }

    pub fn write(&self, base_path: PathBuf) -> Result<()> {
        let string_value =
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).unwrap();
        fs::write(base_path.join(".git/gstack/state.ron"), string_value)?;
        Ok(())
    }
}
