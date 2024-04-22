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
    Add {
        /// Name of this change/branch
        #[arg(short, long)]
        name: Option<String>,
    },
    List {},
}
