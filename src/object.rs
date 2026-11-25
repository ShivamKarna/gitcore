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
