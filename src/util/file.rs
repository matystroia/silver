use std::{
    ffi::OsStr,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    os::unix::fs::MetadataExt,
    path::Path,
    sync::LazyLock,
};

use color_eyre::eyre::Result;
use regex::Regex;

const HASH_BLK_SIZE: usize = 65536;
static EP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[sS]\d+[eE]\d+").unwrap());

pub fn file_hash(file: File) -> Result<String> {
    let fsize = file.metadata()?.size();
    let mut ret = fsize;

    let mut buf = [0; 8];
    let mut reader = BufReader::with_capacity(HASH_BLK_SIZE, file);

    for _ in 0..HASH_BLK_SIZE / 8 {
        reader.read_exact(&mut buf)?;
        ret = ret.wrapping_add(u64::from_ne_bytes(buf));
    }

    reader.seek(SeekFrom::Start(fsize - HASH_BLK_SIZE as u64))?;

    for _ in 0..HASH_BLK_SIZE / 8 {
        reader.read_exact(&mut buf)?;
        ret = ret.wrapping_add(u64::from_ne_bytes(buf));
    }

    Ok(format!("{:01$x}", ret, 16))
}

pub fn is_video(path: &Path) -> bool {
    path.extension()
        .map(OsStr::to_string_lossy)
        .map(|ext| ["mkv", "avi", "mp4"].contains(&ext.as_ref()))
        .unwrap_or(false)
}

pub fn clean_filename(filename: &str) -> Option<String> {
    let mut words = None;
    for sep in ".-_ ".chars() {
        if filename.chars().filter(|&ch| ch == sep).count() >= 3 {
            words = Some(filename.split(sep));
            break;
        }
    }

    let words: Vec<_> = words?.collect();

    if let Some(ep_num_pos) = words.iter().position(|word| EP_RE.is_match(word)) {
        Some(words[..=ep_num_pos].join(" "))
    } else if let Some(year_pos) = words.iter().position(|word| {
        word.parse::<u16>()
            .is_ok_and(|num| num > 1900 && num < 2100)
    }) {
        Some(words[..=year_pos].join(" "))
    } else {
        Some(words.join(" "))
    }
}
