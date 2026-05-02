use anyhow::Context;
use ignore::WalkBuilder;
use std::path::Path;

pub(crate) fn invoke(path: &Path) -> anyhow::Result<()> {
    let mut dir = WalkBuilder::new(path)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|entry| entry.file_name() != ".git")
        // .filter_entry(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .build();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("Failed to open dir:{}", path.display()))?;
        if entry.depth() == 0 || !entry.file_type().map(|f|f.is_file()).unwrap_or(false) {
            continue;
        }
        eprintln!("{:#?}", entry);
    }
    Ok(())
}
