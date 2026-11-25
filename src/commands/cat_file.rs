use anyhow::{Context, Result, bail, ensure};
use flate2::read::ZlibDecoder;
use std::{
    fs::File,
    io::{Read, Write},
};

pub fn execute(pretty_print: bool, object_hash: String) -> Result<()> {
    ensure!(pretty_print, "Only -p mode is currently supported");

    ensure!(
        object_hash.len() == 40 && object_hash.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid object hash"
    );

    let object_hash = object_hash.to_ascii_lowercase();

    let path = format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..]);

    let file = File::open(&path).with_context(|| format!("Failed to open git object: {path}"))?;

    let mut decoder = ZlibDecoder::new(file);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .context("Failed to decompress git object")?;

    let header_end = decompressed
        .iter()
        .position(|&byte| byte == 0)
        .context("Invalid git object: missing NULL byte")?;

    let header =
        std::str::from_utf8(&decompressed[..header_end]).context("Invalid git object header")?;

    let (object_type, size_str) = header
        .split_once(' ')
        .context("Invalid git object header")?;

    let size = size_str.parse::<usize>().context("Invalid object size")?;

    let content = &decompressed[header_end + 1..];

    ensure!(
        content.len() == size,
        "Object size mismatch: expected {size}, got {}",
        content.len()
    );

    match object_type {
        "blob" => {
            std::io::stdout().write_all(content)?;
        }

        _ => {
            bail!("Unsupported object type: {object_type}");
        }
    }

    Ok(())
}
