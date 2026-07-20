mod decoder;
mod renderer;

use std::{fmt, path::Path, sync::Arc};

use crossbeam_queue::ArrayQueue;
use rodio::{OutputStream, OutputStreamBuilder, cpal::BufferSize};

pub use decoder::DecodedAudio;
use renderer::{
    AudioStreams, NO_LOOP_REGION, NO_SEEK, PlaybackState, PreparedRenderState,
    RENDER_PLAN_UPDATE_CAPACITY, RenderPlan, TimelineSource, TransportControl, frame_to_tick,
    pack_loop_region, seconds_to_ticks, tick_to_frame,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopRegion {
    pub start_tick: u32,
    pub end_tick: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportSnapshot {
    pub state: TransportState,
    pub position_tick: u32,
    pub duration_ticks: u32,
    pub loop_region: Option<LoopRegion>,
    pub last_error: Option<String>,
}

pub struct Audio {
    session: Option<PlaybackSession>,
    stopped_position_tick: u32,
    loop_region: Option<LoopRegion>,
    last_error: Option<String>,
    buffer_size: u32,
}

struct PlaybackSession {
    _stream: OutputStream,
    plan: Arc<RenderPlan>,
    transport: Arc<TransportControl>,
    streams: AudioStreams,
    updates: Arc<ArrayQueue<PreparedRenderState>>,
}

impl fmt::Debug for Audio {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Audio")
            .field("has_session", &self.session.is_some())
            .field("stopped_position_tick", &self.stopped_position_tick)
            .field("loop_region", &self.loop_region)
            .field("last_error", &self.last_error)
            .field("buffer_size", &self.buffer_size)
            .finish()
    }
}

impl Default for Audio {
    fn default() -> Self {
        Self::new(crate::settings::DEFAULT_AUDIO_BUFFER_SIZE)
    }
}

impl Audio {
    #[must_use]
    pub(crate) fn new(buffer_size: u32) -> Self {
        Self {
            session: None,
            stopped_position_tick: 0,
            loop_region: None,
            last_error: None,
            buffer_size,
        }
    }

    pub(crate) fn set_buffer_size(&mut self, buffer_size: u32) {
        self.buffer_size = buffer_size;
    }

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
            let (restart_frame, end_frame) =
                self.loop_region
                    .map_or((0, session.plan.duration_frames), |region| {
                        (
                            tick_to_frame(
                                region.start_tick,
                                session.plan.bpm,
                                session.plan.ticks_per_quarter,
                                session.plan.sample_rate,
                            ),
                            tick_to_frame(
                                region.end_tick,
                                session.plan.bpm,
                                session.plan.ticks_per_quarter,
                                session.plan.sample_rate,
                            ),
                        )
                    });
            if session
                .transport
                .frame
                .load(std::sync::atomic::Ordering::Acquire)
                >= end_frame
            {
                session
                    .transport
                    .requested_seek
                    .store(restart_frame, std::sync::atomic::Ordering::Release);
            }
            session.transport.state.store(
                PlaybackState::Playing as u8,
                std::sync::atomic::Ordering::Release,
            );
            return Ok(self.transport());
        }

        let mut stream = OutputStreamBuilder::from_default_device()
            .map(|builder| builder.with_buffer_size(BufferSize::Fixed(self.buffer_size)))
            .and_then(OutputStreamBuilder::open_stream)
            .map_err(|error| CoreError::new(format!("failed to open audio output: {error}")))?;
        stream.log_on_drop(false);
        let sample_rate = stream.config().sample_rate();
        let plan = Arc::new(RenderPlan::build(project, sample_rate));
        let requested_start_frame = tick_to_frame(
            self.stopped_position_tick,
            plan.bpm,
            plan.ticks_per_quarter,
            plan.sample_rate,
        );
        let (loop_start_frame, end_frame) =
            self.loop_region
                .map_or((0, plan.duration_frames), |region| {
                    (
                        tick_to_frame(
                            region.start_tick,
                            plan.bpm,
                            plan.ticks_per_quarter,
                            plan.sample_rate,
                        ),
                        tick_to_frame(
                            region.end_tick,
                            plan.bpm,
                            plan.ticks_per_quarter,
                            plan.sample_rate,
                        ),
                    )
                });
        let start_frame = if requested_start_frame >= end_frame {
            loop_start_frame
        } else {
            requested_start_frame
        };
        let transport = Arc::new(TransportControl {
            state: std::sync::atomic::AtomicU8::new(PlaybackState::Playing as u8),
            frame: std::sync::atomic::AtomicU64::new(start_frame),
            requested_seek: std::sync::atomic::AtomicU64::new(NO_SEEK),
            loop_region: std::sync::atomic::AtomicU64::new(
                self.loop_region.map_or(NO_LOOP_REGION, |region| {
                    pack_loop_region(region.start_tick, region.end_tick)
                }),
            ),
        });
        let mut streams = AudioStreams::default();
        let mut state =
            PreparedRenderState::new(Arc::clone(&plan), &mut streams, true, false, false)?;
        state.prewarm(start_frame);
        let updates = Arc::new(ArrayQueue::new(RENDER_PLAN_UPDATE_CAPACITY));
        let source = TimelineSource::new(state, Arc::clone(&transport), Arc::clone(&updates))?;
        stream.mixer().add(source);
        self.session = Some(PlaybackSession {
            _stream: stream,
            plan,
            transport,
            streams,
            updates,
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
        let transport = self.transport();
        self.session = None;
        self.stopped_position_tick = if transport.state == TransportState::Playing {
            transport.position_tick
        } else {
            self.loop_region.map_or(0, |region| region.start_tick)
        };
        self.transport()
    }

    pub fn seek(&mut self, position_tick: u32) -> TransportSnapshot {
        if let Some(session) = &self.session {
            let maximum_frame = self
                .loop_region
                .map_or(session.plan.duration_frames, |region| {
                    tick_to_frame(
                        region.end_tick,
                        session.plan.bpm,
                        session.plan.ticks_per_quarter,
                        session.plan.sample_rate,
                    )
                });
            let frame = tick_to_frame(
                position_tick,
                session.plan.bpm,
                session.plan.ticks_per_quarter,
                session.plan.sample_rate,
            )
            .min(maximum_frame);
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

    /// Sets or clears the transient A/B playback loop.
    ///
    /// # Errors
    ///
    /// Returns an error when the loop does not have a positive duration.
    pub fn set_loop_region(
        &mut self,
        region: Option<LoopRegion>,
    ) -> Result<TransportSnapshot, CoreError> {
        if region.is_some_and(|region| region.start_tick >= region.end_tick) {
            return Err(CoreError::new("loop end must be after loop start"));
        }
        self.loop_region = region;
        if let Some(session) = &self.session {
            session.transport.loop_region.store(
                region.map_or(NO_LOOP_REGION, |region| {
                    pack_loop_region(region.start_tick, region.end_tick)
                }),
                std::sync::atomic::Ordering::Release,
            );
        }
        Ok(self.transport())
    }

    #[must_use]
    pub fn transport(&self) -> TransportSnapshot {
        let Some(session) = &self.session else {
            return TransportSnapshot {
                state: TransportState::Stopped,
                position_tick: self.stopped_position_tick,
                duration_ticks: 0,
                loop_region: self.loop_region,
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
            loop_region: self.loop_region,
            last_error: self.last_error.clone(),
        }
    }

    pub fn invalidate(&mut self) {
        self.session = None;
        self.stopped_position_tick = 0;
        self.loop_region = None;
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
                timeline.mix_graph.track_routes_to_master(&track.id),
            );
        }
        session
            .plan
            .master
            .update(timeline.master_gain_db, timeline.is_master_muted);
    }

    /// Rebuilds the active render plan without changing transport state or position.
    ///
    /// # Errors
    ///
    /// Returns an error if a newly referenced source cannot be opened.
    pub fn refresh_timeline(
        &mut self,
        project: &PlaybackProject,
        resume_if_at_end: bool,
    ) -> Result<TransportSnapshot, CoreError> {
        let loop_region = self.loop_region;
        let Some(session) = &mut self.session else {
            return Ok(self.transport());
        };
        let current_frame = session
            .transport
            .frame
            .load(std::sync::atomic::Ordering::Acquire);
        let mut next_plan = RenderPlan::build(project, session.plan.sample_rate);
        next_plan.mark_changed_clips(&session.plan);
        let plan = Arc::new(next_plan);
        let remap_position = session.plan.bpm.to_bits() != plan.bpm.to_bits()
            || session.plan.ticks_per_quarter != plan.ticks_per_quarter;
        let prewarm_frame = if remap_position {
            tick_to_frame(
                frame_to_tick(
                    current_frame,
                    session.plan.bpm,
                    session.plan.ticks_per_quarter,
                    session.plan.sample_rate,
                ),
                plan.bpm,
                plan.ticks_per_quarter,
                plan.sample_rate,
            )
        } else {
            current_frame
        };
        let maximum_frame = loop_region.map_or(plan.duration_frames, |region| {
            tick_to_frame(
                region.end_tick,
                plan.bpm,
                plan.ticks_per_quarter,
                plan.sample_rate,
            )
        });
        let prewarm_frame = prewarm_frame.min(maximum_frame);
        let mut state = PreparedRenderState::new(
            Arc::clone(&plan),
            &mut session.streams,
            false,
            remap_position,
            resume_if_at_end,
        )?;
        state.prewarm(prewarm_frame);
        session
            .updates
            .push(state)
            .map_err(|_| CoreError::new("audio render update queue is full"))?;
        session.plan = plan;
        Ok(self.transport())
    }

    #[must_use]
    pub fn duration_ticks(seconds: f64, bpm: f64, ticks_per_quarter: u32) -> u32 {
        seconds_to_ticks(seconds, bpm, ticks_per_quarter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_rewinds_when_transport_is_not_playing() {
        let mut audio = Audio::default();
        audio.seek(960);

        let transport = audio.stop();

        assert_eq!(transport.state, TransportState::Stopped);
        assert_eq!(transport.position_tick, 0);
    }

    #[test]
    fn stop_rewinds_to_loop_start_when_transport_is_not_playing() {
        let mut audio = Audio::default();
        audio
            .set_loop_region(Some(LoopRegion {
                start_tick: 480,
                end_tick: 1_920,
            }))
            .expect("loop region should be valid");
        audio.seek(960);

        let transport = audio.stop();

        assert_eq!(transport.position_tick, 480);
        assert_eq!(
            transport.loop_region,
            Some(LoopRegion {
                start_tick: 480,
                end_tick: 1_920,
            })
        );
    }

    #[test]
    fn rejects_empty_or_reversed_loop_regions() {
        let mut audio = Audio::default();

        assert!(
            audio
                .set_loop_region(Some(LoopRegion {
                    start_tick: 960,
                    end_tick: 960,
                }))
                .is_err()
        );
        assert!(
            audio
                .set_loop_region(Some(LoopRegion {
                    start_tick: 1_920,
                    end_tick: 960,
                }))
                .is_err()
        );
        assert_eq!(audio.transport().loop_region, None);
    }

    #[test]
    fn invalidate_clears_the_loop_region() {
        let mut audio = Audio::default();
        audio
            .set_loop_region(Some(LoopRegion {
                start_tick: 480,
                end_tick: 1_920,
            }))
            .expect("loop region should be valid");

        audio.invalidate();

        assert_eq!(audio.transport().loop_region, None);
    }
}
