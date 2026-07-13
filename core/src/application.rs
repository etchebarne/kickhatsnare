use std::{fmt, path::Path};

use crate::{audio::Audio, library::Library, system::System, workspace::Workspaces};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreError {
    message: String,
}

impl CoreError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CoreError {}

/// Application composition root. Each feature owns its state and operations.
#[derive(Debug)]
pub struct Core {
    audio: Audio,
    library: Library,
    system: System,
    workspaces: Workspaces,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            audio: Audio,
            library: Library::in_memory().expect("in-memory application storage should initialize"),
            system: System,
            workspaces: Workspaces::default(),
        }
    }
}

impl Core {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the application using persistent storage in `data_directory`.
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory or database cannot be initialized.
    pub fn open(data_directory: impl AsRef<Path>) -> Result<Self, CoreError> {
        Ok(Self {
            audio: Audio,
            library: Library::open(data_directory)?,
            system: System,
            workspaces: Workspaces::default(),
        })
    }

    pub fn audio(&mut self) -> &mut Audio {
        &mut self.audio
    }

    pub fn library(&mut self) -> &mut Library {
        &mut self.library
    }

    pub fn system(&mut self) -> &mut System {
        &mut self.system
    }

    pub fn workspaces(&mut self) -> &mut Workspaces {
        &mut self.workspaces
    }
}
