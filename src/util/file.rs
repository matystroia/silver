use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use color_eyre::eyre::{Result, bail};

const HASH_BLK_SIZE: u64 = 65536;

pub fn file_hash(file: File) -> Result<String> {
    let fsize = file.metadata()?.len();

    if fsize < HASH_BLK_SIZE {
        bail!("file too small for opensubtitles moviehash: {fsize} bytes");
    }

    let mut ret = fsize;
    let mut buf = [0; 8];
    let mut reader = BufReader::with_capacity(HASH_BLK_SIZE as usize, file);

    for _ in 0..HASH_BLK_SIZE / 8 {
        reader.read_exact(&mut buf)?;
        ret = ret.wrapping_add(u64::from_le_bytes(buf));
    }

    reader.seek(SeekFrom::Start(fsize - HASH_BLK_SIZE))?;
    for _ in 0..HASH_BLK_SIZE / 8 {
        reader.read_exact(&mut buf)?;
        ret = ret.wrapping_add(u64::from_le_bytes(buf));
    }

    Ok(format!("{ret:016x}"))
}

pub fn is_video(path: &Path) -> bool {
    matches!(path.extension(), Some(val) if val == "mkv" || val == "mp4" || val == "avi")
}

pub fn is_sample(path: &Path) -> bool {
    matches!(path.file_stem(), Some(val) if val == "Sample" || val == "sample")
}
