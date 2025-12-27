use std::io::Read;
use std::io::prelude::*;

use anyhow::Context;

use crate::objects::{self, Kind};

pub(crate) fn invoke(name_only: bool, tree_object: String) -> anyhow::Result<()> {
    // print!("{}, {:?} ", name_only, tree_object);
    let mut object = objects::Object::read(&tree_object)?;
    match object.kind {
        Kind::Tree => {
            let mut read_bytes: u64 = 0;
            let mut mode = Vec::new();
            let mut file_name = Vec::new();
            let mut hashbuf = [0; 20];
            // let stdout = std::io::stdout();
            // let mut stdout = stdout.lock();
            // std::io::copy(&mut object.reader, &mut stdout)?;
            while object.expected_size > read_bytes {
                mode.clear();
                file_name.clear();
                read_bytes += object.reader.read_until(b' ', &mut mode)? as u64;
                read_bytes += object.reader.read_until(0, &mut file_name)? as u64;

                if let Some(&b' ') = mode.last() {
                    mode.pop();
                }
                if let Some(&0) = file_name.last() {
                    file_name.pop();
                }
                object
                    .reader
                    .read_exact(&mut hashbuf)
                    .context("Failed to read the hash")?;
                read_bytes += 20;

                if !name_only {
                    let hash = hex::encode(&hashbuf);
                    let object =
                        objects::Object::read(&hash).context("Failed to identify the hex")?;
                    print!(
                        "{:0>6} {} ",
                        format!("{}", std::str::from_utf8(&mode)?),
                        object.kind,
                    );
                    print!("{}", hash);
                    print!("    ");
                }
                println!("{}", std::str::from_utf8(&file_name)?);
            }
        }
        _ => anyhow::bail!("we do not yet know how to print a '{:?}'", &object.kind),
    }
    Ok(())
}
