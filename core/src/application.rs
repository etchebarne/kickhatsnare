use std::{fmt, path::Path};

use crate::{
    audio::{Audio, TransportSnapshot, TransportState},
    library::Library,
    system::System,
    workspace::{WorkspaceSnapshot, Workspaces},
};

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
            audio: Audio::default(),
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
            audio: Audio::default(),
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

    /// Adds a workspace audio file to the timeline after decoding it.
    ///
    /// # Errors
    ///
    /// Returns an error if the source, track, or audio data is invalid.
    pub fn add_audio_clip(
        &mut self,
        track_id: &str,
        source_path: &str,
        start_tick: u32,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        let resume_if_at_end = self.audio.transport().state == TransportState::Playing;
        let path = self.workspaces.resolve_audio_source(source_path)?;
        let analysis = Audio::analyze(&path)?;
        let timeline = self.workspaces.snapshot()?.timeline;
        let duration_ticks = Audio::duration_ticks(
            analysis.duration_seconds,
            timeline.bpm,
            timeline.ticks_per_quarter,
        );
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Audio");
        let snapshot = self.workspaces.add_audio_clip(
            track_id,
            source_path,
            start_tick,
            name,
            duration_ticks,
            analysis.sample_rate,
            analysis.channels,
            analysis.duration_seconds,
            analysis.waveform,
        )?;
        let project = self.workspaces.playback_project()?;
        self.audio.refresh_timeline(&project, resume_if_at_end)?;
        Ok(snapshot)
    }

    /// Starts or resumes timeline playback.
    ///
    /// # Errors
    ///
    /// Returns an error if sources or the output device cannot be opened.
    pub fn play_audio(&mut self) -> Result<TransportSnapshot, CoreError> {
        let project = self.workspaces.playback_project()?;
        self.audio.play(&project)
    }

    pub fn pause_audio(&mut self) -> TransportSnapshot {
        self.audio.pause()
    }

    pub fn stop_audio(&mut self) -> TransportSnapshot {
        self.audio.stop()
    }

    pub fn seek_audio(&mut self, position_tick: u32) -> TransportSnapshot {
        self.audio.seek(position_tick)
    }

    #[must_use]
    pub fn audio_transport(&self) -> TransportSnapshot {
        self.audio.transport()
    }

    /// Applies current channel and master controls to an active playback session.
    ///
    /// # Errors
    ///
    /// Returns an error if the current workspace snapshot cannot be created.
    pub fn sync_audio_mix(&mut self) -> Result<(), CoreError> {
        let timeline = self.workspaces.snapshot()?.timeline;
        self.audio.sync_mix(&timeline);
        Ok(())
    }

    /// Applies timeline structure changes while preserving active transport state.
    ///
    /// # Errors
    ///
    /// Returns an error if a newly referenced source cannot be opened.
    pub fn refresh_audio_timeline(&mut self) -> Result<(), CoreError> {
        let resume_if_at_end = self.audio.transport().state == TransportState::Playing;
        let project = self.workspaces.playback_project()?;
        self.audio.refresh_timeline(&project, resume_if_at_end)?;
        Ok(())
    }

    pub fn invalidate_audio(&mut self) {
        self.audio.invalidate();
    }
}
