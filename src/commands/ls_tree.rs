use std::io::Read;
use std::io::prelude::*;

use crate::objects::{self, Kind};

pub(crate) fn invoke(name_only: bool, tree_object: String) -> anyhow::Result<()> {
    // print!("{}, {:?} ", name_only, tree_object);
    let mut object = objects::Object::read(&tree_object)?;
    match object.kind {
        Kind::Tree => {
            let mut read_bytes: u64 = 0;
            let mut mode = Vec::new();
            let mut file_name = Vec::new();
            let mut hash = [0; 20];
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
                object.reader.read_exact(&mut hash)?;
                read_bytes += 20;

                if !name_only {
                    print!(
                        "{:0>6} {} ",
                        format!("{}", std::str::from_utf8(&mode)?),
                        Kind::from_mode(&mode)?,
                    );
                    for byte in &hash {
                        print!("{:02x}", byte);
                    }
                    print!("    ");
                }
                println!("{}", std::str::from_utf8(&file_name)?);
            }
        }
        _ => anyhow::bail!("we do not yet know how to print a '{:?}'", &object.kind),
    }
    Ok(())
}
