use anyhow::bail;
use anyhow::Result;
use console::style;
use regex::Regex;
use std::str::FromStr;

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

    ///Force push the current branch to its associated remote, specifying the upstream branch,
    ///but only if there are changes to push
    fn force_push_to_upstream(&self, upstream: &str, upstream_branch: &BranchName) -> Result<()> {
        // Check if there are differences between local and remote branch
        let remote_ref = format!("{}/{}", upstream, upstream_branch);

        // Get the commit hash of the local HEAD
        let local_commit = self.cmd_out(&["rev-parse", "HEAD"])?;
        let local_commit = local_commit.join("").trim().to_string();

        // Try to get the commit hash of the remote branch
        let remote_commit_result = self.cmd_out(&["rev-parse", &remote_ref]);

        // Determine if we need to push
        let need_to_push = match remote_commit_result {
            // Remote branch exists, check if it differs from local
            Ok(remote_commit) => {
                let remote_commit = remote_commit.join("").trim().to_string();

                // Check if local and remote commits are different
                if local_commit != remote_commit {
                    // Check if local is ahead or has diverged from remote
                    let base_commit = self.cmd_out(&["merge-base", "HEAD", &remote_ref])?;
                    let base_commit = base_commit.join("").trim().to_string();

                    // If different and valid ancestry, we should push
                    true
                } else {
                    // Commits are identical, no need to push
                    println!(
                        "No changes to push for branch {}: local and remote are at the same commit",
                        style(upstream_branch).green()
                    );
                    false
                }
            }
            // Remote branch doesn't exist, we should push
            Err(_) => {
                println!(
                    "Remote branch {} doesn't exist yet - will push",
                    style(&remote_ref).yellow()
                );
                true
            }
        };

        // Only push if needed
        if need_to_push {
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
        }

        Ok(())
    }

    fn head_sha(&self, branch_name: &String) -> Result<String> {
        let output = self.cmd_out(&["rev-parse", branch_name])?;
        Ok(output.first().unwrap().clone())
    }
}
