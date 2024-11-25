use std::path::PathBuf;

use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Path to the git repo to operate on
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new stack from the current branch
    New {
        /// Prefix that is applied to all branches in this stack
        #[arg(short, long)]
        prefix: Option<String>,

        /// Name of the intial change/branch
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Stacks a new branch on top of the current stack
    Add {
        /// Name of this change/branch
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Removes the currently checked out branch from the stack
    Remove {},
    /// List all stacks(not in stack branch) or branches(in stack branch)
    #[clap(alias = "ls")]
    List {},
    /// Change to another stack (not in stack branch) or stack branch (in stack branch)
    #[clap(alias = "c")]
    Change {},
    /// Update, auto-rebase, and push all stack branches to make sure they are in sync
    #[clap(alias = "ss")]
    Sync {},
    /// Switch to base branch of the stack
    Base {},
    /// Switch to the above branch of the stack
    Up {},
    /// Switch to the below branch of the stack
    Down {},
    /// Commands related to github PR's
    Pr {
        #[clap(subcommand)]
        cmd: PrCommands,
    },
    /// Delete all stacks and their branches
    Reset {},
}

#[derive(Subcommand)]
pub enum PrCommands {
    /// Create new PR's for all stack branches
    New {},
    #[clap(alias = "ls")]
    /// List open PR's for all stack branches
    List {},
    /// Merge all stack pr's in sequence to the stacks base branch
    Merge {},
}
