use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;

pub struct Logger {
    path: PathBuf,
}

pub static LOGGER: LazyLock<Logger> = LazyLock::new(|| Logger {
    path: PathBuf::from("/home/strigoi/.local/share/silver/log"),
});

impl Logger {
    pub fn log(&self, ln: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .unwrap();
        writeln!(file, "{ln}").unwrap();
    }
}
