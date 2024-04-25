use anyhow::Ok;
use anyhow::Result;
use std::str::FromStr;

use rustygit::{types::BranchName, Repository};

pub trait RepoExtenstions {
    fn current_branch(&self) -> Result<BranchName>;
}

impl RepoExtenstions for Repository {
    fn current_branch(&self) -> Result<BranchName> {
        let branches = self.cmd_out(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(BranchName::from_str(branches.first().unwrap())?)
    }
}
