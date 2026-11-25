use crate::object::{self, ObjectType};
use anyhow::{Context, Result, ensure};

pub fn execute(name_only: bool, hash: String) -> Result<()> {
    let (object_type, content) = object::read_object_via_hash(hash.as_str())?;

    ensure!(
        matches!(object_type, ObjectType::Tree),
        "Expected a tree object, got {object_type:?}"
    );

    let entries = object::parse_tree(&content).context("Failed to parse tree object")?;

    for entry in entries {
        let name = String::from_utf8_lossy(&entry.name);

        if name_only {
            println!("{name}");
            continue;
        }

        let entry_type = match entry.mode.as_str() {
            "40000" | "040000" => "tree",
            "160000" => "commit",
            _ => "blob",
        };

        println!(
            "{} {} {}\t{}",
            entry.mode,
            entry_type,
            entry.hash_hex(),
            name
        );
    }

    Ok(())
}
