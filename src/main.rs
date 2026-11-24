use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::{fs, fs::File, io::Read};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init,

    CatFile {
        #[arg(short, long)]
        pretty_print: bool,

        object_hash: String,
    },
}

// fn decompress(object_hash: &str) -> Result<(Vec<u8>, ObjectType)> {
//     Ok((content.to_vec(), object_type))
// }

enum ObjectType {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEADS", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory");
        }

        Commands::CatFile {
            pretty_print,
            object_hash,
        } => {
            // TODO: shortest unique hash

            // format: blob <size>\0<content>

            anyhow::ensure!(
                pretty_print,
                "mode must be given without -p, (currently modes are not supported!, so just use -p to pretty_print the blob. :) )"
            );

            let path = format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..]);
            let f = File::open(&path)
                .with_context(|| format!("Failed to open git object : {path:?}"))?;

            let mut decoder = ZlibDecoder::new(f);
            let mut decom = Vec::new();

            decoder
                .read_to_end(&mut decom) // read_to_end grows buffer if necessary
                .context("Failed to decompress git object")?;

            let header_end = decom
                .iter()
                .position(|&byte| byte == 0)
                .context("Invalid git object, missing NULL BYTE.")?;

            let header = std::str::from_utf8(&decom[..header_end])
                .context("Git object is not valid Utf-8")?;

            let (object_type, size_str) = header
                .split_once(' ')
                .context("Invalid git object header.")?;

            let object_type = match object_type {
                "blob" => ObjectType::Blob,
                _ => bail!("We do not yet know how to print a '{object_type}'"),
            };

            let size = size_str
                .parse::<usize>()
                .context("Invalid git object header.")?;

            let content_start = header_end + 1;
            let content = &decom[content_start..];

            if content.len() != size {
                bail!(
                    "Object size mismatch, Expected : {size}, got : {}",
                    content.len()
                );
            }

            match object_type {
                ObjectType::Blob => {
                    let result =
                        std::str::from_utf8(&content[..]).context("Invalid result format.")?;
                    println!("{result}");
                }
            }
        }
    }
    Ok(())
}
