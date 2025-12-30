use crate::objects::{Kind, Object};
use anyhow::Context;
use chrono::Local;
use ini::Ini;
use std::{io::Cursor, path::PathBuf};

pub(crate) fn write_commit(
    tree_sha: &str,
    parent_commit_sha: Option<&str>,
    commit_message: &str,
) -> anyhow::Result<[u8; 20]> {
    let mut commit_object = Vec::new();
    commit_object.extend(format!("tree {}\n", tree_sha).as_bytes());
    if let Some(commit) = parent_commit_sha {
        commit_object.extend(format!("parent {}\n", commit).as_bytes());
    }
    // TODO: currently only support the local username email not global
    // if you need to access global try to read the ~/.gitconfig
    let git_config =
        Ini::load_from_file(&PathBuf::from(".git/config")).context("Failed to read git config")?;
    let username = git_config.get_from(Some("user"), "name").unwrap_or("none");
    let email = git_config.get_from(Some("user"), "email").unwrap_or("none");

    let time = Local::now().format("%s %z");
    commit_object.extend(
        format!("author {username} <{email}> {time}\ncommitter {username} <{email}> {time}\n\n")
            .as_bytes(),
    );
    commit_object.extend(format!("{}\n", commit_message).as_bytes());
    let hash = Object {
        kind: Kind::Commit,
        expected_size: commit_object.len() as u64,
        reader: Cursor::new(commit_object),
    }
    .write_to_object()
    .context("Failed to write the commit objects")?;
    Ok(hash)
}

pub(crate) fn invoke(
    tree_sha: String,
    parent_commit_sha: Option<String>,
    commit_message: String,
) -> anyhow::Result<()> {
    let hash = write_commit(&tree_sha, parent_commit_sha.as_deref(), &commit_message)?;
    println!("{}", hex::encode(hash));
    Ok(())
}
