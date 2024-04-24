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
        Some(Commands::New { prefix, name }) => ctx.new_stack(prefix, name)?,
        Some(Commands::Add { name }) => {}
        Some(Commands::List {}) => {}
        None => {}
    }

    println!("Local branches: {:?}", ctx.repo.list_branches()?);

    Ok(())
}

impl GsContext {
    pub fn new_stack(&mut self, prefix: &Option<String>, name: &Option<String>) -> Result<()> {
        let current_branch = self.repo.get_hash(false)?;
        let name = GsContext::get_branch_name(prefix, name)?;
        self.repo
            .create_branch_from_startpoint(&name, current_branch.as_str())?;
        self.repo.switch_branch(&name)?;
        self.state.stacks.push(GitStack {
            prefix: prefix.clone(),
            branches: vec![name.to_string()],
        });

        println!("Created new stack with base branch: {}", name);
        Ok(())
    }

    fn get_branch_name(prefix: &Option<String>, name: &Option<String>) -> Result<BranchName> {
        let prefix_val = prefix.clone().unwrap_or("".to_string());
        let name_val = name.clone().unwrap_or("some-branch".to_string());
        Ok(BranchName::from_str(
            format!("{}-{}", prefix_val.as_str(), name_val.as_str()).as_str(),
        )?)
    }
}
