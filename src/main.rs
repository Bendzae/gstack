use core::panic;
use std::{path::PathBuf, str::FromStr, sync::Arc, thread::current, time::Duration};

use clap::Parser;
use console::{pad_str, style};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use octocrab::{
    models::pulls::PullRequest,
    params::pulls::{MergeMethod, Sort},
    pulls::PullRequestHandler,
    Octocrab,
};
use rustygit::types::BranchName;
use tokio::time::sleep;

use crate::{
    command::{Cli, Commands},
    config::GlobalConfig,
    repo_extensions::RepoExtenstions,
    state::{GitStack, GsState},
};
use anyhow::Result;

mod command;
mod config;
mod repo_extensions;
mod state;

struct GsContext {
    repo: rustygit::Repository,
    base_path: PathBuf,
    github: Arc<Octocrab>,
    state: GsState,
}
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let base_path = PathBuf::from_str(".")?;
    let repo = rustygit::Repository::new(base_path.clone());
    let config = GlobalConfig::read()?;
    let github = Octocrab::builder()
        .personal_token(config.personal_access_token.unwrap())
        .build()?;
    let state = GsState::init(base_path.clone())?;
    let mut ctx = GsContext {
        repo,
        base_path,
        github: Arc::new(github),
        state,
    };

    match &cli.command {
        Some(Commands::New { prefix, name }) => ctx.new_stack(prefix, name)?,
        Some(Commands::Add { name }) => ctx.add_to_stack(name)?,
        Some(Commands::Remove {}) => ctx.remove_current_branch().await?,
        Some(Commands::List {}) => ctx.list()?,
        Some(Commands::Change {}) => ctx.change()?,
        Some(Commands::Sync {}) => ctx.sync(true).await?,
        Some(Commands::Up {}) => ctx.checkout_above()?,
        Some(Commands::Down {}) => ctx.checkout_below()?,
        Some(Commands::Base {}) => ctx.checkout_base()?,
        Some(Commands::Pr { cmd }) => match cmd {
            command::PrCommands::New {} => ctx.create_pull_requests().await?,
            command::PrCommands::List {} => ctx.list_pull_requests().await?,
            command::PrCommands::Merge {} => ctx.merge_pull_requests().await?,
        },
        Some(Commands::Reset {}) => ctx.reset()?,
        None => println!(
            "Welcome to {} version {}! Run {} to see available commands.",
            style("G-Stack").bold().cyan(),
            style(VERSION).bold().yellow(),
            style("gs help").italic().green(),
        ),
    }

    Ok(())
}

impl GsContext {
    fn new_stack(&mut self, prefix: &Option<String>, name: &Option<String>) -> Result<()> {
        let prefix_val = match prefix {
            Some(value) => value.to_string(),
            None => {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Stack Prefix:")
                    .interact_text()
                    .unwrap();
                input
            }
        };
        let name_val = match name {
            Some(value) => value.to_string(),
            None => {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Bottom Branch Name:")
                    .interact_text()
                    .unwrap();
                input
            }
        };
        let current_branch = self.repo.current_branch()?;
        let branch_name = GsContext::get_branch_name(&prefix_val, &name_val)?;
        self.repo
            .create_branch_from_startpoint(&branch_name, current_branch.to_string().as_str())?;
        self.repo.switch_branch(&branch_name)?;
        self.state.stacks.push(GitStack {
            base_branch: current_branch.to_string(),
            prefix: Some(prefix_val.clone()),
            branches: vec![branch_name.to_string()],
        });
        self.state.write(self.base_path.clone())?;

        println!("Created new stack with base branch: {}", branch_name);
        Ok(())
    }

    fn add_to_stack(&mut self, name: &Option<String>) -> Result<()> {
        let name_val = match name {
            Some(value) => value.to_string(),
            None => {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Branch Name:")
                    .interact_text()
                    .unwrap();
                input
            }
        };
        let prefix = self.current_stack().unwrap().prefix.clone().unwrap();
        let name = GsContext::get_branch_name(&prefix, &name_val)?;
        self.current_stack_mut()
            .unwrap()
            .branches
            .push(name.to_string());
        let current_branch = self.repo.current_branch()?;
        self.repo
            .create_branch_from_startpoint(&name, current_branch.to_string().as_str())?;
        self.repo.switch_branch(&name)?;
        self.state.write(self.base_path.clone())?;

        println!("Stacked a new branch with name: {}", name);
        Ok(())
    }

    fn get_branch_name(prefix: &String, name: &String) -> Result<BranchName> {
        Ok(BranchName::from_str(
            format!("{}/{}", prefix.as_str(), name.as_str()).as_str(),
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
            Self::list_stack_branches(stack)?;
        } else {
            Self::list_stacks(&self.state.stacks)?;
        }
        Ok(())
    }

    fn list_stack_branches(stack: &GitStack) -> Result<()> {
        let width = 20;
        for (i, branch) in stack.branches.iter().enumerate().rev() {
            let str = format!("({}): {}", i, style(branch).cyan());
            println!(
                "{}",
                pad_str(str.as_str(), width, console::Alignment::Center, None)
            );
            let str = format!("{}", style("\u{02193}").magenta());
            println!(
                "{}",
                pad_str(str.as_str(), width, console::Alignment::Center, None)
            );
        }
        let str = format!("{}", style(stack.base_branch.clone()).cyan());
        println!(
            "{}",
            pad_str(str.as_str(), width, console::Alignment::Center, None)
        );
        Ok(())
    }

    fn list_stacks(stacks: &Vec<GitStack>) -> Result<()> {
        for (i, stack) in stacks.iter().enumerate() {
            println!("({}): {}", i, style(stack.prefix.clone().unwrap()).cyan());
        }
        Ok(())
    }

    fn change(&self) -> Result<()> {
        if let Some(stack) = self.current_stack() {
            let options: &Vec<String> = &stack
                .branches
                .iter()
                .enumerate()
                .rev()
                .map(|(i, branch)| format!("({}): {}", i, branch))
                .collect();

            let branch_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Stack Branch")
                .default(0)
                .items(options)
                .interact()
                .unwrap();
            let branch = &stack.branches.get(options.len() - branch_idx - 1).unwrap();
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

    async fn sync(&self, update_descriptions: bool) -> Result<()> {
        let current_branch = self.repo.current_branch()?;
        let branches = &self.current_stack().unwrap().branches;
        self.repo.pull_all(branches).ok();
        for (i, branch) in branches.clone().iter().enumerate() {
            let rebase_on = match i {
                0 => &self.current_stack().unwrap().base_branch,
                _ => &branches[i - 1],
            };
            self.repo.rebase(
                BranchName::from_str(branch)?,
                BranchName::from_str(rebase_on)?,
            )?;
            self.repo
                .force_push_to_upstream("origin", &BranchName::from_str(branch)?)?;
        }
        let open_pulls = self.get_pull_requests().await?;
        let remote = self.repo.remote_repo_info()?;
        let pulls = self.github.pulls(remote.owner, remote.name);
        if update_descriptions {
            self.update_pr_descriptions(&pulls, open_pulls).await?;
        }
        self.repo.switch_branch(&current_branch)?;

        Ok(())
    }

    async fn create_pull_requests(&self) -> Result<()> {
        self.sync(false).await?;
        let stack = &self.current_stack().unwrap();
        let branches = &stack.branches;
        let remote = self.repo.remote_repo_info()?;
        let pulls = self.github.pulls(remote.owner, remote.name);
        let open_pulls = self.get_pull_requests().await?;

        let draft = Confirm::new()
            .with_prompt("Create as draft?")
            .interact()
            .unwrap();

        let mut created_pulls = vec![];
        for (i, branch) in branches.iter().enumerate() {
            if let Some(pr) = self.get_branch_pr(&open_pulls, branch) {
                created_pulls.push(pr);
                continue;
            }
            let base = match i {
                0 => &self.current_stack().unwrap().base_branch,
                _ => &branches[i - 1],
            };

            let title = format!(
                "{} (#{}) - {}",
                stack.prefix.clone().unwrap(),
                i,
                branch.split('/').last().unwrap()
            );

            println!("base: {}, title: {}", base, title);
            let pr = pulls
                .create(title, branch, base)
                .draft(draft)
                .body("---")
                .send()
                .await?;
            println!(
                "#{}: {}",
                pr.number,
                style(pr.html_url.clone().unwrap()).blue()
            );
            created_pulls.push(pr);
        }

        self.update_pr_descriptions(&pulls, created_pulls).await?;
        Ok(())
    }

    async fn update_pr_descriptions(
        &self,
        pulls: &PullRequestHandler<'_>,
        prs: Vec<PullRequest>,
    ) -> Result<()> {
        for pr in prs.iter() {
            let mut body = pr
                .body
                .clone()
                .unwrap_or("".to_string())
                .lines()
                .take_while(|line| !line.contains("---"))
                .collect::<Vec<&str>>()
                .join("\n");

            body.push_str("\n---\n");
            prs.iter().rev().for_each(|p| {
                body.push_str(format!("- #{}", p.number).as_str());
                if pr.number == p.number {
                    body.push_str(" (This PR)");
                }
                body.push('\n');
            });
            body = body.clone() + "\n**Created by [gstack](https://github.com/Bendzae/gstack)**";

            pulls.update(pr.number).body(body).send().await?;
            println!("Updated description for PR: #{}", pr.number);
        }
        Ok(())
    }

    async fn list_pull_requests(&self) -> Result<()> {
        let open_pulls = self.get_pull_requests().await?;
        for pr in open_pulls.iter() {
            println!("#{}: {} ", pr.number, pr.html_url.clone().unwrap());
        }
        Ok(())
    }

    async fn get_pull_requests(&self) -> Result<Vec<PullRequest>> {
        let remote = self.repo.remote_repo_info()?;
        let pulls = self.github.pulls(remote.owner, remote.name);
        let open_pulls = pulls
            .list()
            .state(octocrab::params::State::Open)
            .sort(Sort::Created)
            .send()
            .await?
            .items;
        let stack = &self.current_stack().unwrap();
        let branches = &stack.branches;
        let stack_pulls = branches
            .iter()
            .filter_map(|branch| self.get_branch_pr(&open_pulls, branch))
            .collect::<Vec<PullRequest>>();
        Ok(stack_pulls)
    }

    fn get_branch_pr(&self, pull_requests: &[PullRequest], branch: &String) -> Option<PullRequest> {
        pull_requests
            .iter()
            .find(|pr| pr.head.sha == self.repo.head_sha(branch).unwrap_or("".to_string()))
            .cloned()
    }

    fn get_pr_branch(&self, pull_request: &PullRequest) -> Option<String> {
        self.current_stack()?
            .branches
            .iter()
            .find(|branch| {
                pull_request.head.sha == self.repo.head_sha(branch).unwrap_or("".to_string())
            })
            .cloned()
    }

    async fn merge_pull_requests(&mut self) -> Result<()> {
        let remote = self.repo.remote_repo_info()?;
        let github = self.github.clone();
        let pulls = github.pulls(remote.owner, remote.name);
        let open_pulls = self.get_pull_requests().await?;
        let base = self.current_stack().unwrap().base_branch.clone();

        // TODO: fix for Squash and Rebase
        // let merge_method = Select::with_theme(&ColorfulTheme::default())
        //     .with_prompt("Merge method")
        //     .default(0)
        //     .items(&["Squash", "Merge", "Rebase"])
        //     .interact()
        //     .unwrap();
        //
        // let merge_method = match merge_method {
        //     0 => MergeMethod::Squash,
        //     1 => MergeMethod::Merge,
        //     2 => MergeMethod::Rebase,
        //     _ => panic!("Unknown merge method"),
        // };
        let merge_method = MergeMethod::Merge;

        let mut orginal_branches = vec![];
        for pr in &open_pulls {
            pulls.update(pr.number).base(base.clone()).send().await?;
            self.sync(false).await?;
            println!("Merging PR #{}...", pr.number);
            let branch = self.get_pr_branch(pr);
            pulls.merge(pr.number).method(merge_method).send().await?;
            // Not the best way to do this should try to listen to the completed merge somehow
            sleep(Duration::from_secs(3)).await;
            if let Some(branch) = branch {
                orginal_branches.push(branch.clone());
                self.remove_branch_from_stack(&branch)?;
            }
        }

        println!("Sucessfully merged stack!");

        let delete_branches = Confirm::new()
            .with_prompt("Delete local branches?")
            .interact()
            .unwrap();

        if delete_branches {
            for branch in &orginal_branches {
                self.repo.cmd(&["branch", "-d", branch.as_str()])?;
                println!("Deleted branch {}", branch);
            }
        }
        Ok(())
    }

    fn remove_branch_from_stack(&mut self, branch: &String) -> Result<()> {
        println!("Removing branch: {}", branch);
        if !self.current_stack().unwrap().branches.contains(branch) {
            println!("Unknown stack branch, not removing");
            return Ok(());
        }

        let current_branch = self.repo.current_branch()?.to_string().clone();
        let Some(stack_idx) = self
            .state
            .stacks
            .iter_mut()
            .position(|stack| stack.branches.contains(&branch.to_string()))
        else {
            println!("Branch not found, not removing");
            return Ok(());
        };

        let branch_idx = self.state.stacks[stack_idx]
            .branches
            .iter()
            .position(|b| b == branch)
            .unwrap();
        self.state.stacks[stack_idx].branches.remove(branch_idx);

        self.state.write(self.base_path.clone())?;
        // Checkout another stack branch or base if the current branch was deleted
        if &current_branch == branch {
            let base = self.state.stacks[stack_idx].base_branch.clone();
            self.repo.switch_branch(&BranchName::from_str(
                self.state.stacks[stack_idx]
                    .branches
                    .first()
                    .unwrap_or(&base),
            )?)?;
        }

        if self.state.stacks[stack_idx].branches.is_empty() {
            self.state.stacks.remove(stack_idx);
            self.state.write(self.base_path.clone())?;
        }
        println!("Removed branch {}", branch);
        Ok(())
    }

    async fn remove_current_branch(&mut self) -> Result<()> {
        let current = self.repo.current_branch()?;
        self.remove_branch_from_stack(&current.to_string())?;
        let delete_branch = Confirm::new()
            .with_prompt("Delete local branch?")
            .interact()
            .unwrap();

        if delete_branch {
            self.repo
                .cmd(&["branch", "-d", current.to_string().as_str()])?;
            println!("Deleted branch {}", current.to_string());
        }

        self.sync(true).await?;
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
