mod cli;
mod commands;
mod object;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Init => {
            commands::init::execute()?;
        }

        cli::Commands::CatFile {
            pretty_print,
            object_hash,
        } => {
            commands::cat_file::execute(pretty_print, object_hash)?;
        }

        cli::Commands::HashObject { write, file } => {
            commands::hash_object::execute(write, file)?;
        }

        cli::Commands::LsTree {
            name_only,
            tree_hash,
        } => {
            commands::ls_tree::execute(name_only, tree_hash)?;
        }
    }

    Ok(())
}
