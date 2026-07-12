use std::fmt;

use crate::{audio::Audio, system::System, workspace::Workspaces};

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
#[derive(Debug, Default)]
pub struct Core {
    audio: Audio,
    system: System,
    workspaces: Workspaces,
}

impl Core {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn audio(&mut self) -> &mut Audio {
        &mut self.audio
    }

    pub fn system(&mut self) -> &mut System {
        &mut self.system
    }

    pub fn workspaces(&mut self) -> &mut Workspaces {
        &mut self.workspaces
    }
}
