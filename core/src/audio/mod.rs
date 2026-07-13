mod decoder;
mod renderer;

use std::{collections::HashMap, fmt, path::Path, sync::Arc};

use rodio::{OutputStream, OutputStreamBuilder};

pub use decoder::DecodedAudio;
use renderer::{
    NO_SEEK, PlaybackState, RenderPlan, TimelineSource, TransportControl, frame_to_tick,
    seconds_to_ticks, tick_to_frame,
};

use crate::{
    CoreError,
    workspace::{PlaybackProject, TimelineSnapshot},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportSnapshot {
    pub state: TransportState,
    pub position_tick: u32,
    pub duration_ticks: u32,
    pub last_error: Option<String>,
}

#[derive(Default)]
pub struct Audio {
    session: Option<PlaybackSession>,
    stopped_position_tick: u32,
    last_error: Option<String>,
}

struct PlaybackSession {
    _stream: OutputStream,
    plan: Arc<RenderPlan>,
    transport: Arc<TransportControl>,
}

impl fmt::Debug for Audio {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Audio")
            .field("has_session", &self.session.is_some())
            .field("stopped_position_tick", &self.stopped_position_tick)
            .field("last_error", &self.last_error)
            .finish()
    }
}

impl Audio {
    /// Decodes an audio source and returns metadata plus bounded waveform peaks.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be decoded as mono or stereo audio.
    pub fn analyze(path: &Path) -> Result<DecodedAudio, CoreError> {
        decoder::decode(path)
    }

    /// Starts or resumes playback of a project.
    ///
    /// # Errors
    ///
    /// Returns an error if a source cannot be decoded or no output device can be opened.
    pub fn play(&mut self, project: &PlaybackProject) -> Result<TransportSnapshot, CoreError> {
        if let Some(session) = &self.session {
            if session
                .transport
                .frame
                .load(std::sync::atomic::Ordering::Acquire)
                >= session.plan.duration_frames
            {
                session
                    .transport
                    .requested_seek
                    .store(0, std::sync::atomic::Ordering::Release);
            }
            session.transport.state.store(
                PlaybackState::Playing as u8,
                std::sync::atomic::Ordering::Release,
            );
            return Ok(self.transport());
        }

        let mut stream = OutputStreamBuilder::open_default_stream()
            .map_err(|error| CoreError::new(format!("failed to open audio output: {error}")))?;
        stream.log_on_drop(false);
        let sample_rate = stream.config().sample_rate();
        let mut decoded = HashMap::new();
        for path in project
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .map(|clip| &clip.source_path)
        {
            if !decoded.contains_key(path) {
                decoded.insert(path.clone(), Arc::new(decoder::decode(path)?));
            }
        }
        let plan = Arc::new(RenderPlan::build(project, sample_rate, &decoded));
        let start_frame = tick_to_frame(
            self.stopped_position_tick,
            plan.bpm,
            plan.ticks_per_quarter,
            plan.sample_rate,
        )
        .min(plan.duration_frames);
        let transport = Arc::new(TransportControl {
            state: std::sync::atomic::AtomicU8::new(PlaybackState::Playing as u8),
            frame: std::sync::atomic::AtomicU64::new(start_frame),
            requested_seek: std::sync::atomic::AtomicU64::new(NO_SEEK),
        });
        stream.mixer().add(TimelineSource::new(
            Arc::clone(&plan),
            Arc::clone(&transport),
        ));
        self.session = Some(PlaybackSession {
            _stream: stream,
            plan,
            transport,
        });
        self.last_error = None;
        Ok(self.transport())
    }

    pub fn pause(&mut self) -> TransportSnapshot {
        if let Some(session) = &self.session {
            session.transport.state.store(
                PlaybackState::Paused as u8,
                std::sync::atomic::Ordering::Release,
            );
        }
        self.transport()
    }

    pub fn stop(&mut self) -> TransportSnapshot {
        self.session = None;
        self.stopped_position_tick = 0;
        self.transport()
    }

    pub fn seek(&mut self, position_tick: u32) -> TransportSnapshot {
        if let Some(session) = &self.session {
            let frame = tick_to_frame(
                position_tick,
                session.plan.bpm,
                session.plan.ticks_per_quarter,
                session.plan.sample_rate,
            )
            .min(session.plan.duration_frames);
            session
                .transport
                .frame
                .store(frame, std::sync::atomic::Ordering::Release);
            session
                .transport
                .requested_seek
                .store(frame, std::sync::atomic::Ordering::Release);
            let mut snapshot = self.transport();
            snapshot.position_tick = frame_to_tick(
                frame,
                session.plan.bpm,
                session.plan.ticks_per_quarter,
                session.plan.sample_rate,
            );
            return snapshot;
        }
        self.stopped_position_tick = position_tick;
        self.transport()
    }

    #[must_use]
    pub fn transport(&self) -> TransportSnapshot {
        let Some(session) = &self.session else {
            return TransportSnapshot {
                state: TransportState::Stopped,
                position_tick: self.stopped_position_tick,
                duration_ticks: 0,
                last_error: self.last_error.clone(),
            };
        };
        let playback_state = PlaybackState::from_atomic(
            session
                .transport
                .state
                .load(std::sync::atomic::Ordering::Acquire),
        );
        TransportSnapshot {
            state: match playback_state {
                PlaybackState::Stopped => TransportState::Stopped,
                PlaybackState::Playing => TransportState::Playing,
                PlaybackState::Paused => TransportState::Paused,
            },
            position_tick: frame_to_tick(
                session
                    .transport
                    .frame
                    .load(std::sync::atomic::Ordering::Acquire),
                session.plan.bpm,
                session.plan.ticks_per_quarter,
                session.plan.sample_rate,
            ),
            duration_ticks: session.plan.duration_ticks,
            last_error: self.last_error.clone(),
        }
    }

    pub fn invalidate(&mut self) {
        self.session = None;
        self.stopped_position_tick = 0;
    }

    pub fn sync_mix(&mut self, timeline: &TimelineSnapshot) {
        let Some(session) = &self.session else {
            return;
        };
        if session.plan.track_controls.len() != timeline.tracks.len() {
            self.invalidate();
            return;
        }
        for track in &timeline.tracks {
            let Some(control) = session.plan.track_controls.get(&track.id) else {
                self.invalidate();
                return;
            };
            control.update(
                track.gain_db,
                track.pan,
                track.is_muted,
                track.is_soloed,
                track.is_connected,
            );
        }
        session
            .plan
            .master
            .update(timeline.master_gain_db, timeline.is_master_muted);
    }

    #[must_use]
    pub fn duration_ticks(seconds: f64, bpm: f64, ticks_per_quarter: u32) -> u32 {
        seconds_to_ticks(seconds, bpm, ticks_per_quarter)
    }
}
