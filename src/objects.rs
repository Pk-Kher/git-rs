use anyhow::Context;
use core::fmt;
use flate2::read::ZlibDecoder;
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
impl Kind {
    pub(crate) fn from_mode(mode: &[u8]) -> Result<Self, anyhow::Error> {
        match mode {
            b"40000" => Ok(Kind::Tree),
            b"100644" | b"100755" | b"120000" => Ok(Kind::Blob),
            _ => anyhow::bail!("unknown mode:{:?}", std::str::from_utf8(mode)),
        }
    }
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
