use anyhow::Context;
// use flate2::read::ZlibEncoder; // my code
use std::path::Path;

use crate::objects::Object;

pub(crate) fn invoke(write: bool, file_path: &Path) -> anyhow::Result<()> {
    // run test
    // $ mkdir test_dir && cd test_dir
    // $ /path/to/your_program.sh init
    // $ echo "hello world" > test.txt
    // $ ./your_program.sh hash-object -w test.txt
    // 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
    let object = Object::blob_from_file(file_path)?;
    let hash = if write {
        object
            .write_to_object()
            .context("failed to write the blob object")?
    } else {
        object
            .write(std::io::sink())
            .context("failed to write the blob object")?
    };

    println!("{}", hex::encode(hash));
    Ok(())
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
