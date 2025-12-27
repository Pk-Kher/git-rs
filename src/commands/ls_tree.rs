use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::io::prelude::*;
use std::io::{BufReader, Read};

use crate::objects::Kind;

pub(crate) fn invoke(name_only: bool, tree_object: String) -> anyhow::Result<()> {
    // print!("{}, {:?} ", name_only, tree_object);
    let file = fs::File::open(format!(
        ".git/objects/{}/{}",
        &tree_object[..2],
        &tree_object[2..]
    ))
    .with_context(|| format!("Failed to open the file:{:?}", &tree_object))?;
    let z = ZlibDecoder::new(file);
    let mut z = BufReader::new(z);
    let mut buf = Vec::new();
    z.read_until(0, &mut buf)
        .context("Reading header from .git/object")?;
    let header = CStr::from_bytes_with_nul(&buf[..])
        .expect("know there is exactly one nul, and it's at the end");
    let header = header
        .to_str()
        .context(".git/objects file header isn't valid UTF-8")?;
    let Some((kind, size)) = header.split_once(' ') else {
        anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
    };
    let kind = match kind {
        "tree" => Kind::Tree,
        _ => anyhow::bail!("we do not yet know how to print a '{kind}'"),
    };
    let size = size
        .parse::<usize>()
        .context("Failed to parse size from &str to u64")?;
    // let stdout = std::io::stdout();
    // let mut stdout = stdout.lock();
    // std::io::copy(&mut z, &mut stdout)?;
    match kind {
        Kind::Tree => {
            let mut read_bytes: usize = 0;
            let mut mode = Vec::new();
            let mut file_name = Vec::new();
            let mut hash = [0; 20];
            while size > read_bytes {
                mode.clear();
                file_name.clear();

                read_bytes += z.read_until(b' ', &mut mode)?;
                read_bytes += z.read_until(0, &mut file_name)?;

                if let Some(&b' ') = mode.last() {
                    mode.pop();
                }
                if let Some(&0) = file_name.last() {
                    file_name.pop();
                }
                z.read_exact(&mut hash)?;
                read_bytes += 20;

                if !name_only {
                    print!(
                        "{:0>6} {} ",
                        format!("{}", std::str::from_utf8(&mode)?),
                        Kind::from_mode(&mode)?.to_str(),
                    );
                    for byte in &hash {
                        print!("{:02x}", byte);
                    }
                    print!("    ");
                }
                println!("{}", std::str::from_utf8(&file_name)?);
            }
        }
        _ => anyhow::bail!("we do not yet know how to print a '{:?}'", kind),
    }
    Ok(())
}
