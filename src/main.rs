use std::{error::Error, path::PathBuf, str::FromStr};

use clap::{command, Parser, Subcommand};
use rustygit::types::BranchName;
use serde::{Deserialize, Serialize};

use crate::{
    command::{Cli, Commands},
    state::{GitStack, GsState},
};
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
    let base_path = PathBuf::from_str(".")?;
    let repo = rustygit::Repository::new(base_path.clone());
    let state = GsState::init(base_path.clone())?;
    let mut ctx = GsContext {
        repo,
        base_path,
        state,
    };

    match &cli.command {
        Some(Commands::Create { prefix, name }) => {
            let prefix_val = prefix.clone().unwrap_or("".to_string());
            let name_val = name.clone().unwrap_or("some-branch".to_string());
            let current_branch = ctx.repo.get_hash(false)?;
            let name = BranchName::from_str(
                format!("{}-{}", prefix_val.as_str(), name_val.as_str()).as_str(),
            )?;
            ctx.repo
                .create_branch_from_startpoint(&name, current_branch.as_str())?;
            ctx.repo.switch_branch(&name)?;
            ctx.state.stacks.push(GitStack {
                prefix: prefix.clone(),
                branches: vec![name.to_string()],
            });

            println!("Created new stack with base branch: {}", name)
        }
        Some(Commands::Add { name }) => {}
        Some(Commands::List {}) => {}
        None => {}
    }

    println!("Local branches: {:?}", ctx.repo.list_branches()?);

    Ok(())
}
