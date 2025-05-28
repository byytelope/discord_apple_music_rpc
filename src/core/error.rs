use std::fmt;

pub type AppResult<T> = std::result::Result<T, AppError>;

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

macro_rules! impl_from_error {
    ($error_type:ty => $variant:ident) => {
        impl From<$error_type> for AppError {
            fn from(err: $error_type) -> Self {
                AppError::$variant(err.to_string())
            }
        }
    };
    ($error_type:ty => $variant:ident, $format:expr) => {
        impl From<$error_type> for AppError {
            fn from(err: $error_type) -> Self {
                AppError::$variant(format!($format, err))
            }
        }
    };
}

impl_from_error!(std::num::ParseIntError => Parse);
impl_from_error!(std::num::ParseFloatError => Parse);
impl_from_error!(std::io::Error => Io);
impl_from_error!(serde_json::Error => Parse);
impl_from_error!(surf::Error => Network);
impl_from_error!(discord_rich_presence::error::Error => Discord);
impl_from_error!(Box<dyn std::error::Error> => Internal);
impl_from_error!(std::time::SystemTimeError => Internal, "System time error: {}");

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
