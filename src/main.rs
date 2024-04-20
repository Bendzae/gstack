use std::{error::Error, path::PathBuf, str::FromStr};

use clap::{command, Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::{command::Cli, state::GsState};
use anyhow::Result;

mod command;
mod state;

struct GsContext {
    repo: rustygit::Repository,
    base_path: PathBuf,
    state: GsState,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let base_path = PathBuf::from_str("../test-repo")?;
    let repo = rustygit::Repository::new(base_path.clone());
    let state = GsState::init(base_path.clone())?;
    let ctx = GsContext {
        repo,
        base_path,
        state,
    };

    println!("{:?}", ctx.repo.list_branches());

    Ok(())
}
