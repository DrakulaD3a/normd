use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum Action {
    #[command(alias = "n")]
    New { name: Option<String> },

    #[command(alias = "l")]
    List,

    #[command(alias = "f")]
    Find,

    #[command(alias = "v")]
    View { name: Option<String> },

    #[command(alias = "rm")]
    Remove { name: Option<String> },

    #[command(alias = "i")]
    Interactive,

    #[command(alias = "s")]
    Serve { port: Option<u16> },
}

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,

    #[arg(short, long)]
    pub config: Option<PathBuf>,
}
