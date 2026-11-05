//! Unified error type for the whole backend.
//!
//! Every fallible operation returns [`Result<T>`]. `LauncherError` serializes to
//! `{ "kind": "...", "message": "..." }` so the frontend can branch on `kind`
//! without parsing human-readable text.

use std::path::{Path, PathBuf};

use serde::{ser::SerializeStruct, Serialize, Serializer};

/// Crate-wide result alias. The defaulted error parameter keeps
/// `Result<T, OtherError>` usable without shadowing `std::result::Result`.
pub type Result<T, E = LauncherError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error("I/O error at `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("I/O error: {0}")]
    IoRaw(#[from] std::io::Error),

    #[error("{0} is not installed or could not be located")]
    PlatformNotFound(&'static str),

    #[error("registry error: {0}")]
    Registry(String),

    #[error("failed to parse VDF/ACF file `{path}`: {message}")]
    VdfParse { path: PathBuf, message: String },

    #[error("failed to parse manifest `{path}`: {message}")]
    Manifest { path: PathBuf, message: String },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("game `{0}` not found")]
    GameNotFound(String),

    #[error("launch failed: {0}")]
    Launch(String),

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("authentication required: {0}")]
    AuthRequired(&'static str),

    #[error("media capture error: {0}")]
    Capture(String),

    #[error("video encoder error: {0}")]
    Encoder(String),

    #[error("global hotkey error: {0}")]
    Hotkey(String),

    #[error("Windows API error: {0}")]
    WinApi(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not supported: {0}")]
    Unsupported(&'static str),

    #[error("background task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

impl LauncherError {
    /// Stable, machine-readable discriminator for the frontend.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Io { .. } | Self::IoRaw(_) => "io",
            Self::PlatformNotFound(_) => "platformNotFound",
            Self::Registry(_) => "registry",
            Self::VdfParse { .. } => "vdfParse",
            Self::Manifest { .. } => "manifest",
            Self::Json(_) => "json",
            Self::Config(_) => "config",
            Self::GameNotFound(_) => "gameNotFound",
            Self::Launch(_) => "launch",
            Self::Network(_) => "network",
            Self::AuthRequired(_) => "authRequired",
            Self::Capture(_) => "capture",
            Self::Encoder(_) => "encoder",
            Self::Hotkey(_) => "hotkey",
            Self::WinApi(_) => "winApi",
            Self::InvalidInput(_) => "invalidInput",
            Self::Unsupported(_) => "unsupported",
            Self::Task(_) => "task",
        }
    }

    /// Cloneable snapshot, used when the error is broadcast as a Tauri event.
    pub fn payload(&self) -> ErrorPayload {
        ErrorPayload {
            kind: self.kind(),
            message: self.to_string(),
        }
    }
}

impl Serialize for LauncherError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("LauncherError", 2)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub kind: &'static str,
    pub message: String,
}

/// Attach the offending path to `std::io::Error`s.
pub trait IoResultExt<T> {
    fn with_path(self, path: impl AsRef<Path>) -> Result<T>;
}

impl<T> IoResultExt<T> for std::result::Result<T, std::io::Error> {
    fn with_path(self, path: impl AsRef<Path>) -> Result<T> {
        self.map_err(|source| LauncherError::Io {
            path: path.as_ref().to_path_buf(),
            source,
        })
    }
}
