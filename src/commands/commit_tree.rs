use std::io::Cursor;

use anyhow::Context;

use crate::objects::{Kind, Object};

pub(crate) fn invoke(
    tree_sha: String,
    parent_commit: Option<String>,
    commit_message: Option<String>,
) -> anyhow::Result<()> {
    let mut commit_object = Vec::new();
    commit_object.extend(format!("tree {}\n", tree_sha).as_bytes());
    if let Some(commit) = parent_commit {
        commit_object.extend(format!("parent {}\n", commit).as_bytes());
    }
    commit_object.extend(format!("author John Doe <john@example.com> 1234567890 +0000\ncommitter John Doe <john@example.com> 1234567890 +0000").as_bytes());
    if let Some(message) = commit_message {
        commit_object.push(b'\n');
        commit_object.extend(format!("{}", message).as_bytes());
    }
    let hash = Object {
        kind: Kind::Commit,
        expected_size: commit_object.len() as u64,
        reader: Cursor::new(commit_object),
    }
    .write_to_object()
    .context("Failed to write the commit objects")?;
    println!("{}", hex::encode(hash));
    Ok(())
}
