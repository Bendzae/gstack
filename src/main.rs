use std::{error::Error, path::PathBuf, str::FromStr};

use clap::{command, Parser, Subcommand};
use rustygit::types::BranchName;
use serde::{Deserialize, Serialize};

use crate::{
    command::{Cli, Commands},
    repo_extensions::RepoExtenstions,
    state::{GitStack, GsState},
};
use anyhow::Result;

mod command;
mod repo_extensions;
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
        Some(Commands::List {}) => ctx.list(),
        Some(Commands::Up {}) => todo!(),
        Some(Commands::Down {}) => todo!(),
        Some(Commands::Base {}) => ctx.checkout_base()?,
        Some(Commands::Reset {}) => ctx.reset()?,
        None => {}
    }

    Ok(())
}

impl GsContext {
    pub fn new_stack(&mut self, prefix: &Option<String>, name: &Option<String>) -> Result<()> {
        let current_branch = self.repo.current_branch()?;
        let name = GsContext::get_branch_name(prefix, name)?;
        self.repo
            .create_branch_from_startpoint(&name, current_branch.to_string().as_str())?;
        self.repo.switch_branch(&name)?;
        self.state.stacks.push(GitStack {
            base_branch: current_branch.to_string(),
            prefix: prefix.clone(),
            branches: vec![name.to_string()],
        });
        self.state.write(self.base_path.clone())?;

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

    fn current_stack(&self) -> Option<&GitStack> {
        if let Ok(current_branch) = self.repo.current_branch() {
            return self
                .state
                .stacks
                .iter()
                .find(|stack| stack.branches.contains(&current_branch.to_string()));
        };
        None
    }

    fn list(&self) {
        if let Some(stack) = self.current_stack() {
            println!("{:?}", stack)
        } else {
            for (i, stack) in self.state.stacks.iter().enumerate() {
                println!("{:?}: {:?} Base: {:?}", i, stack.prefix, stack.base_branch);
            }
        }
    }

    fn checkout_base(&self) -> Result<()> {
        if let Some(base) = self.current_stack() {
            self.repo
                .switch_branch(&BranchName::from_str(base.base_branch.as_str())?)?;
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        let mut deleted = 0;
        self.state
            .stacks
            .iter()
            .flat_map(|stack| stack.branches.clone())
            .for_each(|branch| {
                deleted += 1;
                self.repo.cmd(&["branch", "-d", branch.as_str()]);
            });
        println!("Deleted {} branches.", deleted);
        self.state = GsState::default();
        self.state.write(self.base_path.clone())?;
        println!("Deleted all stacks and reset state.");
        Ok(())
    }
}
