use crate::objects::{Kind, Object};
use anyhow::Context;
use chrono::Local;
use ini::Ini;
use std::{io::Cursor, path::PathBuf};

// NOTE: it's use to write the commit object
// cargo run -- commit-tree <tree_sha> -p <parent_commit_sha> -m <commit_message>
// it's only write the commit object it will not show any thing in the git log or git show command
pub(crate) fn invoke(
    tree_sha: String,
    parent_commit_sha: Option<String>,
    commit_message: String,
) -> anyhow::Result<()> {
    let hash = write_commit(&tree_sha, parent_commit_sha.as_deref(), &commit_message)?;
    println!("{}", hex::encode(hash));
    Ok(())
}

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
    let git_config = Ini::load_from_file(&PathBuf::from(".git/config"))
        .context("Reading config file. use git config user.name name and user.email email")?;
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
// TEST:  https://app.codecrafters.io/courses/git/stages/jm9
//$ mkdir test_dir && cd test_dir
// $ git init
// Initialized empty Git repository in /path/to/test_dir/.git/
// --------
// # Create a tree, get its SHA
// $ echo "hello world" > test.txt
// $ git add test.txt
// $ git write-tree
// 4b825dc642cb6eb9a060e54bf8d69288fbee4904
// --------
// # Create the initial commit
// $ git commit-tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904 -m "Initial commit"
// 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
// --------
// # Write some changes, get another tree SHA
// $ echo "hello world 2" > test.txt
// $ git add test.txt
// $ git write-tree
// 5b825dc642cb6eb9a060e54bf8d69288fbee4904
// --------
// # Create a new commit with the new tree SHA and parent
// $ git commit-tree 5b825dc642cb6eb9a060e54bf8d69288fbee4904 -p 3b18e512dba79e4c8300dd08aeb37f8e728b8dad -m "Second commit"
// 6c18e512dba79e4c8300dd08aeb37f8e728b8dad
