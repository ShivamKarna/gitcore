use crate::object;
use anyhow::{Context, Result};
use std::{fs, io, path::PathBuf};

pub fn execute(write: bool, file: PathBuf) -> Result<()> {
    let hash = if write {
        let (temp_path, temp_file) = object::create_temp_object()?;

        let hash = match object::hash_blob(&file, temp_file) {
            Ok(hash) => hash,
            Err(error) => {
                let _ = fs::remove_file(&temp_path);
                return Err(error).context("Failed to hash blob");
            }
        };

        let object_dir = format!(".git/objects/{}", &hash[..2]);

        fs::create_dir_all(&object_dir).context("Failed to create object directory")?;

        let destination = PathBuf::from(object_dir).join(&hash[2..]);

        if destination.exists() {
            // The object already exists so no need to write it again.
            fs::remove_file(&temp_path).context("Failed to remove temporary object")?;
        } else {
            fs::rename(&temp_path, &destination)
                .context("Failed to move blob into .git/objects")?;
        }

        hash
    } else {
        object::hash_blob(&file, io::sink()).context("Failed to hash blob")?
    };

    println!("{hash}");

    Ok(())
}
