//! Error enums.

use thiserror::Error;

// TODO: i18n support for all of this

/// An error during the app's lifetime.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
    #[error("Backend error: {0}")]
    Backend(#[from] mau_ui::backend::Error),
    #[error("Clipboard error: {0}")]
    Clipboard(#[from] ClipboardError),
}

/// An error while loading or saving the app's config file.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("config was already loaded in a previous call to load_or_create()")]
    ConfigIsAlreadyLoaded,
}

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("Clipboard content is uninitialized")]
    Uninitialized,
    #[error("Saving to clipboard failed: {error}")]
    SaveFailed { error: String },
    #[error("Clipboard does not contain text")]
    DoesNotContainText,
    #[error("Clipboard does not contain an image")]
    DoesNotContainAnImage,
    #[error("Clipboard content is unavailable")]
    ContentUnavailable,
    #[error("Clipboard is not supported on your platform")]
    NotSupported,
    #[error("Clipboard is occupied by another application. Try again")]
    Occupied,
    #[error("Cannot convert data to/from a clipboard-specific format. Try again or report a bug")]
    ConversionFailed,
    #[error("Unknown clipboard error: {error}")]
    Unknown { error: String },
}

impl From<arboard::Error> for ClipboardError {
    fn from(error: arboard::Error) -> Self {
        match error {
            arboard::Error::ContentNotAvailable => Self::ContentUnavailable,
            arboard::Error::ClipboardNotSupported => Self::NotSupported,
            arboard::Error::ClipboardOccupied => Self::Occupied,
            arboard::Error::ConversionFailure => Self::ConversionFailed,
            arboard::Error::Unknown { description } => Self::Unknown { error: description },
        }
    }
}
