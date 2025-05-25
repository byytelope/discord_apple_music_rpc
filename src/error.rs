use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Discord(String),
    AppleMusic(String),
    Config(String),
    Parse(String),
    Io(String),
    Network(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Discord(msg) => write!(f, "Discord error: {}", msg),
            AppError::AppleMusic(msg) => write!(f, "Apple Music error: {}", msg),
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::Io(msg) => write!(f, "IO error: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::Parse(err.to_string())
    }
}

impl From<std::num::ParseFloatError> for AppError {
    fn from(err: std::num::ParseFloatError) -> Self {
        AppError::Parse(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::Config(err.to_string())
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Internal(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Parse(err.to_string())
    }
}

impl From<surf::Error> for AppError {
    fn from(err: surf::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

impl From<std::time::SystemTimeError> for AppError {
    fn from(err: std::time::SystemTimeError) -> Self {
        AppError::Internal(format!("System time error: {}", err))
    }
}

impl From<discord_presence::error::DiscordError> for AppError {
    fn from(err: discord_presence::error::DiscordError) -> Self {
        AppError::Discord(err.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
