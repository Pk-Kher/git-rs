use anyhow::Context;
use core::fmt;
use flate2::Compression;
use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::io::prelude::*;
use std::path::Path;
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read},
};
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}
impl fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}

impl Object<()> {
    pub(crate) fn blob_from_file(file: impl AsRef<Path>) -> anyhow::Result<Object<impl Read>> {
        let file = file.as_ref();
        let stat = std::fs::metadata(file)
            .with_context(|| format!("Failed to read stat for :{}", file.display()))?;
        let file = std::fs::File::open(file)
            .with_context(|| format!("failed to open the file:{}", file.display()))?;
        Ok(Object {
            kind: Kind::Blob,
            expected_size: stat.len(),
            reader: file,
        })
    }
    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let f = fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..],))
            .context("File does't exits")?;

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
            anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
        };

        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commint" => Kind::Commit,
            _ => anyhow::bail!("we do not yet know how to print a '{kind}'"),
        };
        let size = size.parse::<u64>().context("file size is not valid")?;
        let z = z.take(size);
        Ok(Object {
            kind,
            expected_size: size,
            reader: z,
        })
    }
}
impl<R> Object<R>
where
    R: Read,
{
    pub(crate) fn write(mut self, writer: impl Write) -> anyhow::Result<[u8; 20]> {
        let z = ZlibEncoder::new(writer, Compression::default());
        let mut writer = HashWriter {
            writer: z,
            hasher: Sha1::new(),
        };
        write!(writer, "{} {}\0", self.kind, self.expected_size)?;
        std::io::copy(&mut self.reader, &mut writer).context("Failed to write into the file")?;
        let _ = writer.writer.finish()?;
        let hash = writer.hasher.finalize();
        Ok(hash.into())
    }
    pub(crate) fn write_to_object(self) -> anyhow::Result<[u8; 20]> {
        let tmp = "temporary";
        let hash = self.write(std::fs::File::create(tmp).context("Failed to create File")?)?;
        let raw_hash = hex::encode(hash);
        std::fs::create_dir_all(format!(".git/objects/{}", &raw_hash[..2]))
            .context("Failed to create parent dir for .git/objects/")?;
        std::fs::rename(
            tmp,
            format!(".git/objects/{}/{}", &raw_hash[..2], &raw_hash[2..]),
        )
        .context("Failed to move tmp file into .git/objects")?;
        Ok(hash)
    }
}
pub(crate) struct HashWriter<W> {
    pub(crate) writer: W,
    pub(crate) hasher: Sha1,
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
