use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::MetadataExt,
    path::Path,
};

use anyhow::Context;
use sha1::{Digest, Sha1};

use crate::objects::Object;
const READER_VERSION: u32 = 2;
pub(crate) fn invoke(add: bool, file_path: Option<String>) -> anyhow::Result<()> {
    eprintln!("{} {:?} ", add, file_path);
    if let Some(file_path) = file_path {
        let mut buf: Vec<u8> = Vec::with_capacity(128);
        // header
        buf.extend(b"DIRC");
        buf.extend(READER_VERSION.to_be_bytes());
        buf.extend((1 as u32).to_be_bytes());
        // entry start;
        let metadata = fs::metadata(&file_path).context("Reading metadata for the file")?;
        // all the meatadata if the value is greater then u32::MAX it will get truncated
        buf.extend((metadata.ctime() as u32).to_be_bytes());
        buf.extend((metadata.ctime_nsec() as u32).to_be_bytes());
        buf.extend((metadata.mtime() as u32).to_be_bytes());
        buf.extend((metadata.mtime_nsec() as u32).to_be_bytes());
        buf.extend((metadata.dev() as u32).to_be_bytes());
        buf.extend((metadata.ino() as u32).to_be_bytes());
        buf.extend(metadata.mode().to_be_bytes());
        buf.extend(metadata.uid().to_be_bytes());
        buf.extend(metadata.gid().to_be_bytes());
        buf.extend((metadata.size() as u32).to_be_bytes());
        let object = Object::blob_from_file(&file_path)?;
        let sha1 = object
            .write(std::io::sink())
            .context("Create the hash of the blob")?;
        buf.extend(sha1);
        // NOTE: you need to handle the merge conflict related stage
        let flag = build_flag(0, file_path.as_bytes().len()); // here we don't need string length but we need bytes len
        buf.extend(flag.to_be_bytes());
        buf.extend(file_path.as_bytes());
        buf.extend(b"\0");
        // one file total length should be multiply of 8
        let padding = vec![0; (8 - (buf.len() % 8)) % 8];
        buf.extend(padding);
        // entry end;
        let mut hasher = Sha1::new();
        hasher.update(&buf);
        let hash = hasher.finalize();
        buf.extend(hash);
        write_atomic_index(buf)?;
    }
    Ok(())
}
fn build_flag(stage: u16, path_length: usize) -> u16 {
    let stage = stage & 0b11; // enforce 2 bits
    let path_length = std::cmp::min(path_length as u16, 0x0FFF); // enforce 12 bits
    (stage << 12) | path_length // we shift 12 bit of the stage so it will make room for the path
    // length
}

fn write_atomic_index(buf: Vec<u8>) -> anyhow::Result<()> {
    let index_dir = Path::new(".git");
    let index_lock_file = index_dir.join("index.lock");
    let index_file = index_dir.join("index");
    {
        let mut tmp = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(index_lock_file)
            .context("Create the .git/index.lock file.")?;
        tmp.write_all(&buf[..])
            .context("Write the .git/index.lock file.")?;
        tmp.sync_all().context("Sync all .git/index.lock file.")?;
    }
    fs::rename(".git/index.lock", index_file)
        .context("Rename the .git/index.lock file to .git/index file")?;
    Ok(())
    // NOTE:
    // We use `OpenOptions` instead of `File::open` / `File::create` because writing
    // `.git/index` must be done atomically and exclusively.
    //
    // - `OpenOptions::create_new(true)` maps to `O_CREAT | O_EXCL` and guarantees
    //   that `.git/index.lock` is created ONLY if it does not already exist.
    //   This prevents concurrent writers and avoids race conditions.
    //
    // - The index is written fully to `index.lock`, NOT directly to `index`.
    //   Writing directly risks partial writes and repository corruption.
    //
    // - `sync_all()` (fsync) is required to force file contents + metadata
    //   from the OS page cache to stable storage before renaming.
    //   Without this, a crash or power loss could leave a corrupted index.
    //
    // Correct order (same as Git core):
    //   write → fsync(file) → rename(index.lock → index) → fsync(.git dir)
    //
    // This guarantees atomicity, durability, and corruption-free updates.
}

// NOTE: you need to read byte by byte first 12 is the header
// DIRC 4 bytes
// version 4 bytes
// entry 4 bytes
// entries must be sorted by path bytes;
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
// padding = (8 - (entry_size_raw % 8)) % 8
// path_length = 8
// raw_size = 62 + 9 = 71
// 71 % 8 = 7
// padding = 1
// padding is \0 null bytes
// flag - 2 bytes 12 bits
// bit index: 15 14 13 12 11 .............. 0
//            ┌─┬──┬──┬───────────────────┐
//            │ │  │  │
//            │ │  │  └─ path length (12 bits)
//            │ │  └──── stage (2 bits)
//            │ └─────── extended / assume-valid
//            └──────── unused / future
// 15        14        13        12        11                   0
// ┌─────────┬─────────┬─────────┬─────────┬────────────────────┐
// │ unused  │ ext/AV  │ stage 1 │ stage 0 │   path length (12) │
// └─────────┴─────────┴─────────┴─────────┴────────────────────┘
// Are you in a merge conflict?
// ├─ NO → stage = 0
// └─ YES
//    ├─ base version → stage = 1
//    ├─ current branch → stage = 2
//    └─ incoming branch → stage = 3
//
