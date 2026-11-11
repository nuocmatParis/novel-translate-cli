#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    pub fn label(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockLogEntry {
    pub id: usize,
    pub timestamp: String,
    pub level: LogLevel,
    pub event: String,
    pub volume: String,
    pub chapter: String,
    pub model: String,
    pub status: String,
    pub message: String,
}
