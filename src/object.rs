use anyhow::{Context, Result};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub struct WriterAndHasher<W>
where
    W: Write,
{
    writer: W,
    hasher: Sha1,
}

impl<W: Write> Write for WriterAndHasher<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;

        self.hasher.update(&buf[..n]);

        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

pub fn create_temp_object() -> Result<(PathBuf, File)> {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    loop {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        let path =
            PathBuf::from(".git/objects").join(format!("gitcore-{}-{id}.tmp", std::process::id()));

        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),

            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                continue;
            }

            Err(error) => {
                return Err(error).with_context(|| {
                    format!("Failed to create temporary object: {}", path.display())
                });
            }
        }
    }
}

pub fn hash_blob<W>(path: &Path, output: W) -> Result<String>
where
    W: Write,
{
    let mut input =
        File::open(path).with_context(|| format!("Failed to open: {}", path.display()))?;

    let metadata = input
        .metadata()
        .with_context(|| format!("Failed to stat: {}", path.display()))?;

    let encoder = ZlibEncoder::new(output, Compression::default());

    let mut writer = WriterAndHasher {
        writer: encoder,
        hasher: Sha1::new(),
    };

    // format : blob <size>\0
    // streaming the git OBJECT HEADER for hash and compression.
    write!(writer, "blob {}\0", metadata.len())?;

    // streaming the FILE CONTENT for hash and compression.
    std::io::copy(&mut input, &mut writer)?;

    let WriterAndHasher {
        writer: encoder,
        hasher,
    } = writer;

    let _output = encoder.finish()?;

    let hash = hasher.finalize();

    Ok(hex::encode(hash))
}

pub struct TreeEntry {
    pub mode: String,
    pub name: Vec<u8>,
    pub hash: [u8; 20], // 20 byte Sha1 hash
}

impl TreeEntry {
    // This converts the 20-byte hash into the familiar 40-character SHA-1 representation
    pub fn hash_hex(&self) -> String {
        self.hash.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

// format: <mode> <filename>\0<20-byte SHA-1>

// for example,(conceptual) :
// 100644 README.md\0[20 raw hash bytes]
// 40000 src\0[20 raw hash bytes]
pub fn parse_tree(content: &[u8]) -> anyhow::Result<Vec<TreeEntry>> {
    let mut cursor = 0;
    let mut entries = Vec::<TreeEntry>::new();

    while cursor < content.len() {
        let mode_end = content
            .get(cursor..)
            .context("Invalid tree: invalid cursor")?
            .iter()
            .position(|&byte| byte == b' ')
            .context("Invalid tree: missing mode separator")?
            + cursor;

        // dbg!(cursor);
        // dbg!(mode_end);
        // dbg!(&content[cursor..mode_end]);
        // dbg!(String::from_utf8_lossy(&content[cursor..mode_end]));
        //
        // dbg!(content.len());
        // dbg!(&content[..content.len().min(50)]);

        let mode = std::str::from_utf8(
            content
                .get(cursor..mode_end)
                .context("Invalid tree entry mode")?,
        )
        .context("Invalid tree entry mode")?
        .to_owned();

        let name_start = mode_end + 1;

        let name_end = content
            .get(name_start..)
            .context("Invalid tree entry name")?
            .iter()
            .position(|&byte| byte == 0)
            .context("Invalid tree entry: missing NULL byte")?
            + name_start;

        let name = content
            .get(name_start..name_end)
            .context("Invalid tree entry name")?
            .to_vec();

        let hash_start = name_end + 1;
        let hash_end = hash_start.checked_add(20).context("Hash offset overflow")?;

        let hash_bytes = content
            .get(hash_start..hash_end)
            .context("Invalid tree entry hash")?;

        let hash: [u8; 20] = hash_bytes
            .try_into()
            .context("Invalid tree entry hash length")?;

        entries.push(TreeEntry { mode, name, hash });
        cursor = hash_end;
    }

    Ok(entries)
}

#[derive(Debug)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

pub fn read_object_via_hash(hash: &str) -> anyhow::Result<(ObjectType, Vec<u8>)> {
    use anyhow::{Context, bail, ensure};
    use flate2::read::ZlibDecoder;
    use std::fs::File;
    use std::io::Read;

    ensure!(
        hash.len() == 40 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid object hash"
    );

    let hash = hash.to_ascii_lowercase();

    let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);

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

    let content = decompressed[header_end + 1..].to_vec();

    ensure!(
        content.len() == size,
        "Object size mismatch: expected {size}, got {}",
        content.len()
    );

    let object_type = match object_type {
        "blob" => ObjectType::Blob,
        "tree" => ObjectType::Tree,
        "commit" => ObjectType::Commit,
        _ => bail!("Unsupported object type: {object_type}"),
    };

    Ok((object_type, content))
}
