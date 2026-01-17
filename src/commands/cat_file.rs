use anyhow::Context;

use crate::objects::{self, Kind};

// NOTE: it's use to read the blob object file "cat-file -p hash" it will uncompress the file
// content
// run test
// /path/to/your_program.sh cat-file -p 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
// hello world
pub(crate) fn invoke(pretty_print: bool, object_hash: String) -> anyhow::Result<()> {
    anyhow::ensure!(pretty_print, "Please provide the flag 'p'");
    let mut object = objects::Object::read(&object_hash)?;
    match object.kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n =
                std::io::copy(&mut object.reader, &mut stdout).context("Write into the stdout")?;
            anyhow::ensure!(
                n == object.expected_size,
                ".git/object file was not expected size (expected :{}, actual: {n})",
                object.expected_size
            );
        }
        _ => anyhow::bail!("we do not yet know how to print a '{:?}'", object.kind),
    }
    Ok(())
}

// TEST:
// $ mkdir /tmp/test_dir && cd /tmp/test_dir
// $ /path/to/your_program.sh init
// $ echo "hello world" > test.txt # The tester will use a random string, not "hello world"
// $ git hash-object -w test.txt
// 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
