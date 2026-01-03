use std::io::{BufRead, Cursor, Read};

use anyhow::Context;

// NOTE:: this command is use to read the .git/index file
pub(crate) fn invoke() -> anyhow::Result<()> {
    // NOTE: .git/index file get store as binary
    //
    let f = std::fs::read(".git/index").context("Open the .git/index file.")?;
    let mut reader = Cursor::new(&f[..]);
    let mut header = [0u8; 4];
    reader
        .read_exact(&mut header)
        .context("Reading the header")?;
    assert_eq!(&header[..], b"DIRC");

    let _ = read_be(&mut reader).context("Reading version of the .git/index")?;
    let num_of_entries = read_be(&mut reader).context("Reading entry from the .git/index")?;

    for i in 0..num_of_entries {
        let mut stats = [0u8; 62];
        reader
            .read_exact(&mut stats)
            .with_context(|| format!("Reading the stats for {} entry", i))?;
        let mut file_path = Vec::new();
        let file_path_bytes_count = reader
            .read_until(0, &mut file_path)
            .with_context(|| format!("Reading file path for entry {}", i))?;
        if file_path.contains(&b'\0') {
            file_path.pop();
        }
        let padding = (8 - ((62 + file_path_bytes_count) % 8)) % 8;
        let mut padding = vec![0u8; padding];
        reader
            .read_exact(&mut padding)
            .context("Reading padding bytes")?;
        println!("{}", str::from_utf8(&file_path)?)
    }

    // let stdout = std::io::stdout();
    // let mut stdout = stdout.lock();
    // stdout.write_all(&header)?;
    Ok(())
}

fn read_be(r: &mut Cursor<&[u8]>) -> anyhow::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}
// you need to read byte by byte first 12 is the header
// DIRC 4 bytes
// version 4 bytes
// entry 4 bytes
// | Field      | Size (bytes) |  // all the number will get store in the  <big endian>
// | ---------- | ------------ |
// | ctime_sec  | 4            |
// | ctime_nsec | 4            |
// | mtime_sec  | 4            |
// | mtime_nsec | 4            |
// | dev        | 4            |
// | ino        | 4            |
// | mode       | 4            |
// | uid        | 4            |
// | gid        | 4            |
// | file_size  | 4            |
// | sha1       | 20           |
// | flags      | 2            |
// | path       | N            |
// | path       |              |
// | ends with  | 1
// | \0         |
// if the total size of each entry is not multiple of 8 the needs to add the padding
//padding = (8 - (entry_size_raw % 8)) % 8
// path_length = 8
// raw_size = 62 + 9 = 71
// 71 % 8 = 7
// padding = 1
// padding is \0 null bytes
