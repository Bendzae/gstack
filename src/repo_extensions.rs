use anyhow::Ok;
use anyhow::Result;
use std::str::FromStr;

use rustygit::{types::BranchName, Repository};

pub trait RepoExtenstions {
    fn current_branch(&self) -> Result<BranchName>;
    fn rebase(&self, branch: BranchName, on: BranchName) -> Result<()>;
}

impl RepoExtenstions for Repository {
    fn current_branch(&self) -> Result<BranchName> {
        let branches = self.cmd_out(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(BranchName::from_str(branches.first().unwrap())?)
    }

    fn rebase(&self, branch: BranchName, on: BranchName) -> Result<()> {
        self.switch_branch(&branch)?;
        let output = self.cmd_out(&["rebase", "--update-refs", on.to_string().as_str()])?;
        println!("{:?}", output);
        Ok(())
    }
}
