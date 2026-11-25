use anyhow::{Result, ensure};
use std::io::Write;

use crate::object;
use crate::object::ObjectType;

pub fn execute(pretty_print: bool, object_hash: String) -> Result<()> {
    ensure!(pretty_print, "Only -p mode is currently supported");

    let (object_type, content) = object::read_object_via_hash(&object_hash)?;

    match object_type {
        ObjectType::Blob => {
            std::io::stdout().write_all(&content)?;
        }

        _ => {
            anyhow::bail!("Unsupported object type for cat-file: {object_type:?}");
        }
    }

    Ok(())
}
