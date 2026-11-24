use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init,

    HashObject {
        file: PathBuf,

        #[arg(short, long)]
        write: bool,
    },

    CatFile {
        object: String,

        #[arg(short, long)]
        pretty_print: bool,
    },

    LsTree {
        tree_hash: String,
    },

    WriteTree,

    CommitTree {
        tree_hash: String,

        #[arg(short, long)]
        parent: Option<String>,

        #[arg(short, long)]
        message: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("This is the INIT COMMAND ");
        }

        Commands::HashObject { file, write } => {
            println!("File : {file:?}, Write : {write}");
        }

        Commands::CatFile {
            object,
            pretty_print,
        } => {
            println!("Object : {object}, \nPretty_Print : {pretty_print}");
        }

        Commands::LsTree { tree_hash } => {
            println!("Hash  : {tree_hash}")
        }

        Commands::WriteTree => {
            println!("Write Tree!");
        }

        Commands::CommitTree {
            tree_hash,
            parent,
            message,
        } => {
            println!("Tree: {tree_hash}");
            match parent {
                Some(hash) => println!("Parent : {hash}"),
                None => println!("This is the root commit."),
            }
            println!("Message: {message}");
        }
    }
}
