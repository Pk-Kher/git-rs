use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::io::prelude::*;
use std::io::{BufReader, Read};

use crate::objects::Kind;

pub(crate) fn invoke(pretty_print: bool, object_hash: String) -> anyhow::Result<()> {
    // run test
    // /path/to/your_program.sh cat-file -p 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
    // hello world
    anyhow::ensure!(pretty_print, "Please provide the flag 'p'");
    let f = fs::File::open(format!(
        ".git/objects/{}/{}",
        &object_hash[..2],
        &object_hash[2..],
    ))
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
        _ => anyhow::bail!("we do not yet know how to print a '{:?}'", kind),
    }
    Ok(())
}
