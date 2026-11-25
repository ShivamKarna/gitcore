use anyhow::{Context, bail, ensure};
use clap::{Parser, Subcommand};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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

enum ObjectType {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            fs::create_dir_all(".git/objects")?;
            fs::create_dir_all(".git/refs")?;

            fs::write(".git/HEAD", "ref: refs/heads/main\n")?;

            println!("Initialized git directory");
        }

        Commands::CatFile {
            pretty_print,
            object_hash,
        } => {
            ensure!(pretty_print, "Only -p mode is currently supported");

            ensure!(
                object_hash.len() == 40 && object_hash.bytes().all(|b| b.is_ascii_hexdigit()),
                "Invalid object hash"
            );

            let path = format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..]);

            let file =
                File::open(&path).with_context(|| format!("Failed to open git object: {path}"))?;

            let mut decoder = ZlibDecoder::new(file);
            let mut decompressed = Vec::new();

            decoder
                .read_to_end(&mut decompressed)
                .context("Failed to decompress git object")?;

            let header_end = decompressed
                .iter()
                .position(|&byte| byte == 0)
                .context("Invalid git object: missing NULL byte")?;

            let header = std::str::from_utf8(&decompressed[..header_end])
                .context("Invalid git object header")?;

            let (object_type, size_str) = header
                .split_once(' ')
                .context("Invalid git object header")?;

            let object_type = match object_type {
                "blob" => ObjectType::Blob,
                _ => bail!("Unsupported object type: {object_type}"),
            };

            let size = size_str.parse::<usize>().context("Invalid object size")?;

            let content = &decompressed[header_end + 1..];

            ensure!(
                content.len() == size,
                "Object size mismatch: expected {size}, got {}",
                content.len()
            );

            match object_type {
                ObjectType::Blob => {
                    std::io::stdout().write_all(content)?;
                }
            }
        }

        Commands::HashObject { write, file } => {
            let hash = if write {
                let (temp_path, temp_file) = create_temp_object()?;

                let hash = write_blob(&file, temp_file).context("Failed to write blob object")?;

                let object_dir = format!(".git/objects/{}", &hash[..2]);

                fs::create_dir_all(&object_dir).context("Failed to create object directory")?;

                let destination = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);

                fs::rename(&temp_path, &destination)
                    .context("Failed to move blob into .git/objects")?;

                hash
            } else {
                write_blob(&file, std::io::sink()).context("Failed to hash blob object")?
            };

            println!("{hash}");
        }
    }

    Ok(())
}

fn write_blob<W>(path: &Path, output: W) -> anyhow::Result<String>
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

struct WriterAndHasher<W>
where
    W: Write,
{
    writer: W,
    hasher: Sha1,
}

impl<W> Write for WriterAndHasher<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;

        self.hasher.update(&buf[..n]);

        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

// Creates a unique temporary file inside .git/objects.
fn create_temp_object() -> anyhow::Result<(PathBuf, File)> {
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
                    format!("Failed to create temporary file: {}", path.display())
                });
            }
        }
    }
}
