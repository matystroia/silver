use std::path::Path;

use walkdir::WalkDir;

pub fn dir_size(path: &Path) -> color_eyre::Result<u64> {
    let mut size = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file()
            && let Ok(metadata) = entry.metadata()
        {
            size += metadata.len();
        }
    }
    Ok(size)
}

pub fn human_size(size: u64) -> String {
    let (label, unit) = [
        ("GB", 1024 * 1024 * 1024),
        ("MB", 1024 * 1024),
        ("KB", 1024),
        ("B", 1),
    ]
    .into_iter()
    .find(|(_, sz)| size > *sz)
    .unwrap_or(("B", 1));

    format!("{:.1} {label}", size as f64 / unit as f64)
}
