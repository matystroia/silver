use std::{fmt::Display, fs::OpenOptions, io::Write, path::PathBuf, sync::LazyLock};

use crate::core::Config;

static LOG_PATH: LazyLock<PathBuf> = LazyLock::new(|| Config::data_dir().join("log.txt"));

enum LogLevel {
    Debug,
    Info,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}
pub struct Logger {}

impl Logger {
    fn log(level: LogLevel, ln: impl Display) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*LOG_PATH)
            .unwrap();
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "{timestamp} [{level}] {ln}").unwrap();
    }

    pub fn debug(ln: impl Display) { Self::log(LogLevel::Debug, ln); }
    pub fn info(ln: impl Display) { Self::log(LogLevel::Info, ln); }
    pub fn error(ln: impl Display) { Self::log(LogLevel::Error, ln); }
}
