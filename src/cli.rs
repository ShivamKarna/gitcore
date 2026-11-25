use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "gitcore")]
#[command(about = "A Git implementation in Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init,

    CatFile {
        #[arg(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },

    HashObject {
        #[arg(short = 'w')]
        write: bool,

        file: PathBuf,
    },
}
