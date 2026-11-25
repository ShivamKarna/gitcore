use anyhow::{Context, Result};
use std::fs;

pub fn execute() -> Result<()> {
    fs::create_dir_all(".git/objects").context("Failed to create .git/objects")?;

    fs::create_dir_all(".git/refs").context("Failed to create .git/refs")?;

    fs::write(".git/HEAD", "ref: refs/heads/main\n").context("Failed to create HEAD")?;

    println!("Initialized git directory");

    Ok(())
}
