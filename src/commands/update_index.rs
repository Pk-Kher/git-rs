use std::{
    fs::{self, OpenOptions},
    io::{BufReader, Read, Write,BufRead},
    os::unix::fs::MetadataExt,
    path::Path,
};

use anyhow::Context;
use sha1::{Digest, Sha1};

use crate::{
    commands::ls_file::{read_file_path, read_index_header},
    objects::Object,
};
const READER_VERSION: u32 = 2;
// NOTE: currently i'm just creating new file every time i need to update file if it's already exist
// you need to store the files in the some specific order (Index entries are sorted lexicographically by pathname (byte order))
pub(crate) fn invoke(_add: bool, file_path: Option<String>) -> anyhow::Result<()> {
    // read the existing index file
    let file_path = file_path.unwrap_or_else(|| {
        eprintln!(
            "No file path provided. Use `--add <file>` to specify a file to add to the index."
        );
        std::process::exit(1);
    });
    let file = std::fs::File::open(".git/index");
    // header
    let mut header_buf = Vec::with_capacity(8);
    header_buf.extend(b"DIRC");
    header_buf.extend(READER_VERSION.to_be_bytes()); // version
    let mut count_entries: u32 = 0;
    // let count_entries= (0 as u32).to_be_bytes();

    let mut buf: Vec<u8> = Vec::with_capacity(128);
    // NOTE: check for the delete file as well
    if file.is_ok() {
        let mut reader = BufReader::new(file.unwrap());
        let num_of_entries = read_index_header(&mut reader)?;
        count_entries = num_of_entries;
        let mut entry_file_path_buf = Vec::with_capacity(256);
        let mut stats = [0u8; 62];
        for i in 0..num_of_entries {
            entry_file_path_buf.clear();
            // read the stats for each entry
            reader
                .read_exact(&mut stats)
                .with_context(|| format!("Reading the stats for {} entry", i))?;
            let entry_padding=read_file_path(&mut reader, &mut entry_file_path_buf)?;
            let entry_file_path_str = str::from_utf8(&entry_file_path_buf)?;
            if entry_file_path_str == &file_path {
                eprintln!("entry file path: {:?}", str::from_utf8(&entry_file_path_buf)?);
                write_index_file(&file_path, &mut buf)?;
            }else{
                buf.extend(stats); 
                buf.extend(&entry_file_path_buf[..]);
                buf.extend(b"\0");
                let padding = vec![0; entry_padding];
                buf.extend(padding)
            }
        }
        // if let Some(file_path) = file_path{
        //     write_index_file(file_path,&mut buf)?;
        // }
        header_buf.extend((count_entries).to_be_bytes());
        header_buf.extend(buf);
        write_atomic_index(header_buf)?;
    }
    Ok(())
}
fn build_flag(stage: u16, path_length: usize) -> u16 {
    let stage = stage & 0b11; // enforce 2 bits
    let path_length = std::cmp::min(path_length as u16, 0x0FFF); // enforce 12 bits
    (stage << 12) | path_length // we shift 12 bit of the stage so it will make room for the path
    // length
}

fn write_index_file(file_path: &String, buf: &mut Vec<u8>) -> anyhow::Result<()> {
    // entry start;
    let metadata = fs::metadata(file_path).context("Reading metadata for the file")?;
    // all the metadata if the value is greater then u32::MAX it will get truncated
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
    // end metadata
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
    Ok(())
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
