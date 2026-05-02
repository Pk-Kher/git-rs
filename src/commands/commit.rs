use crate::commands::{commit_tree::write_commit, write_tree::write_tree_for};
use anyhow::Context;
use std::path::{ PathBuf};

// NOTE: it will add your latest commit to the list
// it will create the new commit object
// it will update the HEAD ref
// so when you run git log you will see the new commit
// and show child based on the parent commit
// cargo run -- commit -m "commit message"
pub fn invoke(message: &str) -> anyhow::Result<()> {
    // to commit we need value same as the commit-tree
    //value of head=ref: refs/heads/master
    let head_ref = std::fs::read_to_string(".git/HEAD")
        .with_context(|| format!("Failed to read the head "))?;
    let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
        anyhow::bail!("Refusing to commit onto detached HEAD")
    };
    let head_ref = head_ref.trim();
    // here might get some problem what if head_ref is availble but we are failed to read it
    let parent_sha = std::fs::read_to_string(format!("./.git/{head_ref}"));
    let parent_sha = match &parent_sha {
        Ok(parent_sha) => Some(parent_sha.trim()),
        Err(_) => None,
    };

    // NOTE: i think there is something wrong here if i specifiy the . as tree path then it will
    // have whole files in the tree. so i don't know this right or wrong
    let Some(tree_hash) = write_tree_for(&PathBuf::from("."))? else {
        eprintln!("Not commiting the empty tree");
        return Ok(());
    };
    let tree_hash = hex::encode(tree_hash);

    let commit_hash = write_commit(&tree_hash, parent_sha, &message)
        .with_context(|| format!("Failed to generate commit hash"))?;
    let commit_hash = hex::encode(commit_hash);

    write_branch(head_ref, &commit_hash)?;
    eprintln!("HEAD is now at {}", commit_hash);

    Ok(())
}

pub(crate) fn write_branch(head_ref: &str, commit_hash: &str) -> anyhow::Result<()> {
    let path = format!(".git/{}", head_ref);
    std::fs::write(path, commit_hash)?;
    Ok(())
}
