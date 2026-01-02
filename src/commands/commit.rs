use crate::commands::{commit_tree::write_commit, write_tree::write_tree_for};
use anyhow::Context;
use std::path::PathBuf;

pub fn invoke(message: &str) -> anyhow::Result<()> {
    // to commit we need value same as the commit-tree
    //value of head=ref: refs/heads/master
    let head_ref = std::fs::read_to_string(".git/HEAD")
        .with_context(|| format!("Failed to read the head "))?;
    let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
        anyhow::bail!("Refusing to commit onto detached HEAD")
    };
    let head_ref = head_ref.trim();
    let parent_sha = std::fs::read_to_string(format!("./.git/{head_ref}"))
        .with_context(|| format!("Failed to read the parent commit hash:{}", head_ref))?;
    let parent_sha = parent_sha.trim();

    let Some(tree_hash) = write_tree_for(&PathBuf::from("."))? else {
        eprintln!("Not commiting the empty tree");
        return Ok(());
    };
    let tree_hash = hex::encode(tree_hash);

    let commit_hash = write_commit(&tree_hash, Some(parent_sha), &message)
        .with_context(|| format!("Failed to generate commit hash"))?;
    let commit_hash = hex::encode(commit_hash);

    std::fs::write(format!(".git/{head_ref}"), &commit_hash)
        .with_context(|| format!("Failed to update the HEAD ref at :{}", head_ref))?;
    eprintln!("HEAD is now at {}", commit_hash);
    Ok(())
}
