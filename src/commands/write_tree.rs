use anyhow::Context;
use ignore::WalkBuilder;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::{cmp::Ordering, path::Path};

use crate::objects::{Kind, Object};
pub(crate) fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    // let mut dir = std::fs::read_dir(path).context("Failed to read the current dir")?;
    let mut dir = WalkBuilder::new(path)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .max_depth(Some(1))
        .filter_entry(|entry| entry.file_name() != ".git")
        .build();
    let mut entries = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("Failed to open dir:{}", path.display()))?;
        if entry.depth() == 0 {
            continue;
        }
        let name = entry.file_name().to_os_string();
        let metadata = entry
            .metadata()
            .with_context(|| format!("Failed to get metadta for dir:{}", path.display()))?;
        entries.push((entry, name, metadata));
    }

    entries.sort_unstable_by(|a, b| {
        let afn = &a.1;
        let afn = afn.as_encoded_bytes();
        let bfn = &b.1;
        let bfn = bfn.as_encoded_bytes();
        let common_len = std::cmp::min(afn.len(), bfn.len());
        match afn[..common_len].cmp(&bfn[..common_len]) {
            Ordering::Equal => {}
            o => return o,
        }
        if afn.len() == bfn.len() {
            return Ordering::Equal;
        }
        let c1 = if let Some(c) = afn.get(common_len).copied() {
            Some(c)
        } else if a.2.is_dir() {
            Some(b'/')
        } else {
            None
        };
        let c2 = if let Some(c) = bfn.get(common_len).copied() {
            Some(c)
        } else if b.2.is_dir() {
            Some(b'/')
        } else {
            None
        };
        c1.cmp(&c2)
    });

    let mut tree_object = Vec::new();
    for (entry, name, metadata) in entries {
        let mode = if metadata.is_dir() {
            "40000"
        } else if metadata.is_symlink() {
            "120000"
        } else if (metadata.permissions().mode() & 0o111) != 0 {
            // has at least one executable bit set
            "100755"
        } else {
            "100644"
        };
        let path = entry.path();
        // println!("path:{:?}-{}", path, metadata.is_dir());
        let hash = if metadata.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                continue;
            };
            hash
        } else {
            let tmp = "tmp";
            let object = Object::blob_from_file(&path)?;
            let hash = object
                .write(std::fs::File::create(tmp).context("Failed to create the tree file object")?)
                .context("failed to write the blob object")?;
            // let stdout = std::io::stdout();
            // let mut stdout = stdout.lock();
            // std::io::copy(&mut hash, &mut stdout);
            let hax_encode = hex::encode(hash);
            std::fs::create_dir_all(format!(".git/objects/{}/", &hax_encode[..2]))
                .context("Failed to create the tree dir")?;
            std::fs::rename(
                tmp,
                format!(".git/objects/{}/{}", &hax_encode[..2], &hax_encode[..2]),
            )
            .context("Failed to rename tree file")?;
            hash
        };
        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }
    if tree_object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            Object {
                kind: Kind::Tree,
                expected_size: tree_object.len() as u64,
                reader: Cursor::new(tree_object),
            }
            .write_to_object()
            .context("Failed to write a tree object")?,
        ))
    }
}

pub(crate) fn invoke(path: &Path) -> anyhow::Result<()> {
    let Some(hash) = write_tree_for(path).context("Faild construct root tree object")? else {
        anyhow::bail!("asked to make tree object for empty tree");
    };
    println!("{}", hex::encode(hash));
    Ok(())
}
