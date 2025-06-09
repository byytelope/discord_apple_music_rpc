use std::fmt;

pub type PipeBoomResult<T> = std::result::Result<T, PipeBoomError>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PipeBoomError {
    /// Errors due to the Apple Music app
    AppleMusic(String),
    /// Errors from Discord Rich Presence
    Discord(String),
    /// Errors from the Osascript interface
    Osascript(String),
    /// Errors in configuration
    Config(String),
    /// Errors in parsing data
    Parse(String),
    /// IO errors
    Io(String),
    /// Network errors
    Network(String),
    /// General internal errors
    Internal(String),
    /// Errors in IPC implementation
    Ipc(String),
    /// Errors during setup/uninstallation
    Setup(String),
}

impl fmt::Display for PipeBoomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PipeBoomError::AppleMusic(msg) => write!(f, "APPLE MUSIC ERROR: {}", msg),
            PipeBoomError::Discord(msg) => write!(f, "DISCORD ERROR: {}", msg),
            PipeBoomError::Osascript(msg) => write!(f, "OSASCRIPT ERROR: {}", msg),
            PipeBoomError::Config(msg) => write!(f, "CONFIGURATION ERROR: {}", msg),
            PipeBoomError::Parse(msg) => write!(f, "PARSE ERROR: {}", msg),
            PipeBoomError::Io(msg) => write!(f, "IO ERROR: {}", msg),
            PipeBoomError::Network(msg) => write!(f, "NETWORK ERROR: {}", msg),
            PipeBoomError::Internal(msg) => write!(f, "INTERNAL ERROR: {}", msg),
            PipeBoomError::Ipc(msg) => {
                write!(f, "IPC ERROR: {}", msg)
            }
            PipeBoomError::Setup(msg) => {
                write!(f, "SETUP ERROR: {}", msg)
            }
        }
    }
}

impl PipeBoomError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            PipeBoomError::Discord(_) | PipeBoomError::AppleMusic(_) | PipeBoomError::Network(_)
        )
    }
}

impl std::error::Error for PipeBoomError {}

macro_rules! impl_from_error {
    ($error_type:ty => $variant:ident) => {
        impl From<$error_type> for PipeBoomError {
            fn from(err: $error_type) -> Self {
                PipeBoomError::$variant(err.to_string())
            }
        }
    };
    ($error_type:ty => $variant:ident, $format:expr) => {
        impl From<$error_type> for PipeBoomError {
            fn from(err: $error_type) -> Self {
                PipeBoomError::$variant(format!($format, err))
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
impl_from_error!(std::time::SystemTimeError => Internal, "SYSTEM TIME ERROR: {}");

impl From<&str> for PipeBoomError {
    fn from(err: &str) -> Self {
        PipeBoomError::Internal(err.to_string())
    }
}

impl From<String> for PipeBoomError {
    fn from(err: String) -> Self {
        PipeBoomError::Internal(err)
    }
}
