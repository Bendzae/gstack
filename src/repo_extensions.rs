use anyhow::Result;
use anyhow::{bail, Ok};
use console::style;
use regex::Regex;
use std::fmt::Debug;
use std::{ops::Rem, str::FromStr};

use rustygit::{types::BranchName, Repository};

pub struct RemoteRepoInfo {
    pub owner: String,
    pub name: String,
}

pub trait RepoExtenstions {
    fn current_branch(&self) -> Result<BranchName>;
    fn rebase(&self, branch: BranchName, on: BranchName) -> Result<()>;
    fn pull_all(&self, branches: &Vec<String>) -> Result<()>;
    fn remote_repo_url(&self) -> Result<String>;
    fn remote_repo_info(&self) -> Result<RemoteRepoInfo>;
    fn force_push_to_upstream(&self, upstream: &str, upstream_branch: &BranchName) -> Result<()>;
    fn head_sha(&self, branch_name: &String) -> Result<String>;
}

impl RepoExtenstions for Repository {
    fn current_branch(&self) -> Result<BranchName> {
        let branches = self.cmd_out(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(BranchName::from_str(branches.first().unwrap())?)
    }

    fn rebase(&self, branch: BranchName, on: BranchName) -> Result<()> {
        self.switch_branch(&branch)?;
        let output = self.cmd_out(&["rebase", "--update-refs", on.to_string().as_str()])?;
        println!(
            "Rebased branch {} on {} with output: {:?}",
            style(branch).green(),
            style(on).green(),
            style(output.join(",")).white().on_black()
        );
        Ok(())
    }

    fn pull_all(&self, branches: &Vec<String>) -> Result<()> {
        for branch in branches {
            self.switch_branch(&BranchName::from_str(branch.as_str())?)?;
            let output = self.cmd_out(&["pull", "--rebase"])?;
            println!(
                "Pulled branch {} with output: {:?}",
                style(branch).green(),
                style(output.join(",")).white().on_black()
            );
        }
        Ok(())
    }

    fn remote_repo_url(&self) -> Result<String> {
        let output = self.cmd_out(&["config", "--get", "remote.origin.url"])?;
        if (output.is_empty()) {
            bail!("No remote found");
        }
        Ok(output.first().unwrap().clone())
    }

    fn remote_repo_info(&self) -> Result<RemoteRepoInfo> {
        let url = self.remote_repo_url()?;
        let re = Regex::new(r"(https://github.com/|git@github.com:)([^/]+)/([^/]+)\.git").unwrap();

        if let Some(captures) = re.captures(url.as_str()) {
            Ok(RemoteRepoInfo {
                owner: captures.get(2).unwrap().as_str().to_string(),
                name: captures.get(3).unwrap().as_str().to_string(),
            })
        } else {
            bail!("Malformed remote url")
        }
    }

    ///Force push the curent branch to its associated remote, specifying the upstream branch
    fn force_push_to_upstream(&self, upstream: &str, upstream_branch: &BranchName) -> Result<()> {
        let output = self.cmd_out(&[
            "push",
            "-u",
            upstream,
            upstream_branch.to_string().as_str(),
            "--force-with-lease",
        ])?;
        println!(
            "Force pushed to upstream branch {} with output: {:?}",
            style(upstream_branch).green(),
            style(output.join(",")).white().on_black()
        );
        Ok(())
    }

    fn head_sha(&self, branch_name: &String) -> Result<String> {
        let output = self.cmd_out(&["rev-parse", branch_name])?;
        Ok(output.first().unwrap().clone())
    }
}
