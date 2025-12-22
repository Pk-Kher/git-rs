use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
// use flate2::read::ZlibEncoder; // my code
use sha1::{Digest, Sha1};
use std::ffi::CStr;
use std::fs;
use std::io::prelude::*;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
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
        file_path: PathBuf,
    },
}
enum Kind {
    Blob,
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Commands::CatFile {
            pretty_print: p,
            object_hash,
        } => {
            // run test
            // /path/to/your_program.sh cat-file -p 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
            // hello world
            anyhow::ensure!(p, "Please provide the flag 'p'");
            let f = fs::File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..],
            ))
            .context(" file does't exits")?;
            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);
            let mut buf = Vec::new();
            z.read_until(0, &mut buf)
                .context("Reading header from .git/object")?;
            let header = CStr::from_bytes_with_nul(&buf)
                .expect("know there is exactly one nul, and it's at the end");
            let header = header
                .to_str()
                .context(".git/objects file header isn't valid UTF-8")?;
            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!(
                    ".git/objects file header did not start with a known type: '{header}'"
                );
            };
            let kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("we do not yet know how to print a '{kind}'"),
            };
            let size = size.parse::<u64>().context("file size is not valid")?;

            let mut z = z.take(size);
            match kind {
                Kind::Blob => {
                    let stdout = std::io::stdout();
                    let mut stdout = stdout.lock();
                    let n = std::io::copy(&mut z, &mut stdout).context("Write into the stdout")?;
                    anyhow::ensure!(
                        n == size,
                        ".git/object file was not expected size (expected :{size}, actual: {n})"
                    );
                }
            }
        }
        Commands::HashObject { write, file_path } => {
            // run test
            // $ mkdir test_dir && cd test_dir
            // $ /path/to/your_program.sh init
            // $ echo "hello world" > test.txt
            // $ ./your_program.sh hash-object -w test.txt
            // 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
            fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
            where
                W: Write,
            {
                let stat = std::fs::metadata(&file)
                    .with_context(|| format!("Reading metadata of the file {:?}", &file))?;
                let z = ZlibEncoder::new(writer, Compression::default());
                let mut writer = HashWriter {
                    writer: z,
                    hasher: Sha1::new(),
                };
                write!(writer, "blob ")?;
                write!(writer, "{}\0", stat.len())?;
                let mut file = std::fs::File::open(&file)?;
                std::io::copy(&mut file, &mut writer)
                    .context("Merge the file content and the header")?;
                writer.writer.finish()?;
                let hash = writer.hasher.finalize();
                Ok(format!("{:x}", hash))
            }
            let hash = if write {
                let tmp = "temporary";
                let hash = write_blob(
                    &file_path,
                    std::fs::File::create(tmp).context("Failed to create File")?,
                )?;
                std::fs::create_dir_all(format!(".git/objects/{}", &hash[..2]))
                    .context("Failed to create parent dir for .git/objects/")?;
                std::fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
                    .context("Failed to move tmp file into .git/objects")?;
                hash
            } else {
                write_blob(
                    &file_path,
                    std::fs::File::create(&file_path).context("Failed to create File")?,
                )?
            };
            println!("{}", hash);
            //
            //
            // this is my code  its working but not optimize
            // let f = fs::File::open(file_path).context("File is not exits")?;
            // let mut b = BufReader::new(f);
            // let mut buffer = Vec::new();
            // b.read_to_end(&mut buffer)
            //     .context("Read the encoded file")?;
            // // add the bolb <size>\0<content>
            // let size = buffer.len();
            // let mut header = format!("blob {}", size).into_bytes();
            // header.push(0);
            // let mut new_buf = Vec::with_capacity(size + header.len());
            // new_buf.extend_from_slice(&header);
            // new_buf.extend_from_slice(&buffer);
            //
            // // calculate the hash
            // let mut hasher = Sha1::new();
            // hasher.update(&new_buf);
            // let result = format!("{:x}", hasher.finalize());
            // println!("hash:{result}");
            // let mut z = ZlibEncoder::new(&new_buf[..], Compression::fast());
            // buffer.clear();
            // z.read_to_end(&mut buffer).context("some")?;
            //
            // fs::create_dir_all(format!(".git/objects/{}", &result[..2]))
            //     .context("Failed to create parent dir for .git/objects/")?;
            // fs::write(
            //     format!(".git/objects/{}/{}", &result[..2], &result[2..]),
            //     &buffer[..],
            // )
            // .context("Failed to create file in .git/objects")?;
        }
    }
    Ok(())
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
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
