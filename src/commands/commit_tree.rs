use crate::objects::{Kind, Object};
use anyhow::Context;
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn invoke(
    tree_sha: String,
    parent_commit_sha: Option<String>,
    commit_message: String,
) -> anyhow::Result<()> {
    let mut commit_object = Vec::new();
    commit_object.extend(format!("tree {}\n", tree_sha).as_bytes());
    if let Some(commit) = parent_commit_sha {
        commit_object.extend(format!("parent {}\n", commit).as_bytes());
    }
    let now = SystemTime::now();
    let time = now.duration_since(UNIX_EPOCH)?.as_secs();
    commit_object.extend(format!("author pk-kher <pradipkher3@gmail.com> 1766904475 +0530\ncommitter pk-kher <pradipkher3@gmail.com> 1766904475 +0530\n\n").as_bytes());
    commit_object.extend(format!("{}\n", commit_message).as_bytes());
    let hash = Object {
        kind: Kind::Commit,
        expected_size: commit_object.len() as u64,
        reader: Cursor::new(commit_object),
    }
    .write_to_object()
    .context("Failed to write the commit objects")?;
    println!("{} {}", hex::encode(hash), time);
    Ok(())
}
