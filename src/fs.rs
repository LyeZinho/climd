use std::path::PathBuf;

use crate::app::FileEntry;

pub fn discover_md_files(dir: &PathBuf) -> Result<Vec<FileEntry>, std::io::Error> {
    let mut entries = Vec::new();

    let read_dir = std::fs::read_dir(dir)?;
    for entry in read_dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            entries.push(FileEntry::Directory(
                path.file_name().unwrap().to_string_lossy().to_string(),
                path,
            ));
        } else if path.extension().map(|ext| ext == "md").unwrap_or(false) {
            entries.push(FileEntry::File(
                path.file_name().unwrap().to_string_lossy().to_string(),
                path,
            ));
        }
    }

    entries.sort_by(|a, b| match (a, b) {
        (FileEntry::Directory(_, _), FileEntry::File(_, _)) => std::cmp::Ordering::Less,
        (FileEntry::File(_, _), FileEntry::Directory(_, _)) => std::cmp::Ordering::Greater,
        _ => a.name().cmp(b.name()),
    });

    Ok(entries)
}
