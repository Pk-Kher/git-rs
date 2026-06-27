use anyhow::Context;
use ignore::WalkBuilder;
use std::path::Path;

use crate::commands::{
    update_index::{IndexEntry, update_index_file, update_index_file_buffer},
};

pub(crate) fn invoke(path: &Path) -> anyhow::Result<()> {
    let mut dir = WalkBuilder::new(path)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|entry| entry.file_name() != ".git")
        // .filter_entry(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .build();

    let mut index_file = std::fs::File::open(".git/index");
    let mut buf: Vec<IndexEntry> = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("Failed to open dir:{}", path.display()))?;
        if !entry.file_type().map(|f| f.is_file()).unwrap_or(false) {
            continue;
        }
        update_index_file_buffer(&mut index_file, entry.path(), &mut buf).with_context(
            || format!("Faield to update the your selected files into the index file"),
        )?;
    }
    update_index_file(buf)?;
    Ok(())
}
