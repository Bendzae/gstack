use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use rustygit::types::BranchName;

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
        Some(Commands::Add { name }) => ctx.add_to_stack(name)?,
        Some(Commands::List {}) => ctx.list()?,
        Some(Commands::Change {}) => ctx.change()?,
        Some(Commands::Up {}) => ctx.checkout_above()?,
        Some(Commands::Down {}) => ctx.checkout_below()?,
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

    fn add_to_stack(&mut self, name: &Option<String>) -> Result<()> {
        let current_branch = self.repo.current_branch()?;
        let prefix = &self.current_stack().unwrap().prefix;
        let name = GsContext::get_branch_name(prefix, name)?;
        self.current_stack_mut()
            .unwrap()
            .branches
            .push(name.to_string());
        self.repo
            .create_branch_from_startpoint(&name, current_branch.to_string().as_str())?;
        self.repo.switch_branch(&name)?;
        self.state.write(self.base_path.clone())?;

        println!("Stacked a new branch with name: {}", name);
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

    fn current_stack_mut(&mut self) -> Option<&mut GitStack> {
        if let Ok(current_branch) = self.repo.current_branch() {
            return self
                .state
                .stacks
                .iter_mut()
                .find(|stack| stack.branches.contains(&current_branch.to_string()));
        };
        None
    }

    fn current_stack_position(&self) -> Option<(&GitStack, usize)> {
        if let Ok(current_branch) = &self.repo.current_branch() {
            for stack in &self.state.stacks {
                let branch_idx = stack
                    .branches
                    .iter()
                    .position(|branch| branch == &current_branch.to_string());
                if let Some(idx) = branch_idx {
                    return Some((&stack, idx));
                }
            }
        };
        None
    }

    fn list(&self) -> Result<()> {
        if let Some(stack) = self.current_stack() {
            println!("{:?}", stack);
        } else {
            println!("{:?}", self.state.stacks);
        }
        Ok(())
    }

    fn change(&self) -> Result<()> {
        if let Some(stack) = self.current_stack() {
            let branch_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Stack Branch")
                .default(0)
                .items(&stack.branches)
                .interact()
                .unwrap();
            let branch = &stack.branches.get(branch_idx).unwrap();
            self.repo.switch_branch(&BranchName::from_str(branch)?)?;
        } else {
            let stacks: Vec<String> = self
                .state
                .stacks
                .iter()
                .enumerate()
                .map(|(i, stack)| format!("({}): {}", i, stack.prefix.clone().unwrap()))
                .collect();

            let stack_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Stack")
                .default(0)
                .items(&stacks)
                .interact()
                .unwrap();

            println!("Moving to {}!", stacks[stack_idx]);
            let selected_stack = self.state.stacks.get(stack_idx).unwrap();
            let branch = selected_stack.branches.first().unwrap();

            self.repo.switch_branch(&BranchName::from_str(branch)?)?;
        }
        Ok(())
    }

    fn checkout_base(&self) -> Result<()> {
        if let Some(stack) = self.current_stack() {
            self.repo
                .switch_branch(&BranchName::from_str(stack.base_branch.as_str())?)?;
        }
        Ok(())
    }

    fn checkout_above(&self) -> Result<()> {
        if let Some((stack, idx)) = self.current_stack_position() {
            if let Some(branch) = stack.branches.get(idx + 1) {
                self.repo.switch_branch(&BranchName::from_str(branch)?)?;
            }
        }
        Ok(())
    }

    fn checkout_below(&self) -> Result<()> {
        if let Some((stack, idx)) = self.current_stack_position() {
            if let Some(branch) = stack.branches.get(idx - 1) {
                self.repo.switch_branch(&BranchName::from_str(branch)?)?;
            }
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
