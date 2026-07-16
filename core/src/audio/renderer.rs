use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use arc_swap::ArcSwap;
use crossbeam_queue::ArrayQueue;
use rodio::{Decoder, Source};

use crate::{
    CoreError,
    workspace::{ClipStretchMode, PlaybackClip, PlaybackProject},
};

pub const NO_SEEK: u64 = u64::MAX;

const DECODE_BLOCK_FRAMES: u64 = 4_096;
const DECODE_CACHE_BLOCKS: usize = 64;
const DECODE_PREFETCH_BLOCKS: u64 = 3;
const DECODE_REQUEST_CAPACITY: usize = 64;
const RETIRED_PLAN_CAPACITY: usize = 8;
pub const RENDER_PLAN_UPDATE_CAPACITY: usize = 8;
const PREWARM_TIMEOUT: Duration = Duration::from_millis(250);
const STRETCH_GRAIN_HOP_SECONDS: f64 = 0.02;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlaybackState {
    Stopped = 0,
    Playing = 1,
    Paused = 2,
}

impl PlaybackState {
    pub fn from_atomic(value: u8) -> Self {
        match value {
            1 => Self::Playing,
            2 => Self::Paused,
            _ => Self::Stopped,
        }
    }
}

#[derive(Debug)]
pub struct TransportControl {
    pub state: AtomicU8,
    pub frame: AtomicU64,
    pub requested_seek: AtomicU64,
}

#[derive(Debug)]
pub struct TrackControl {
    muted: AtomicBool,
    soloed: AtomicBool,
    connected: AtomicBool,
    left_gain: AtomicU32,
    right_gain: AtomicU32,
}

#[derive(Debug)]
pub struct MasterControl {
    muted: AtomicBool,
    gain: AtomicU32,
}

#[derive(Debug)]
pub struct RenderPlan {
    pub sample_rate: u32,
    pub bpm: f64,
    pub ticks_per_quarter: u32,
    pub duration_frames: u64,
    pub duration_ticks: u32,
    pub tracks: Vec<RenderTrack>,
    pub track_controls: HashMap<String, Arc<TrackControl>>,
    pub master: Arc<MasterControl>,
}

#[derive(Debug)]
pub struct RenderTrack {
    control: Arc<TrackControl>,
    clips: Vec<RenderClip>,
}

#[derive(Debug)]
pub struct RenderClip {
    id: String,
    source_path: PathBuf,
    start_frame: u64,
    end_frame: u64,
    source_offset_seconds: f64,
    source_duration_seconds: f64,
    source_rate: f64,
    stretch_mode: ClipStretchMode,
    pitch_ratio: f64,
    left_gain: f32,
    right_gain: f32,
    fade_in_at_start: bool,
    fade_out_at_end: bool,
    fade_in_on_activation: bool,
}

impl TrackControl {
    fn new(gain_db: f64, pan: f64, muted: bool, soloed: bool, connected: bool) -> Self {
        let (left, right) = channel_gains(gain_db, pan);
        Self {
            muted: AtomicBool::new(muted),
            soloed: AtomicBool::new(soloed),
            connected: AtomicBool::new(connected),
            left_gain: AtomicU32::new(left.to_bits()),
            right_gain: AtomicU32::new(right.to_bits()),
        }
    }

    pub fn update(&self, gain_db: f64, pan: f64, muted: bool, soloed: bool, connected: bool) {
        let (left, right) = channel_gains(gain_db, pan);
        self.left_gain.store(left.to_bits(), Ordering::Relaxed);
        self.right_gain.store(right.to_bits(), Ordering::Relaxed);
        self.muted.store(muted, Ordering::Relaxed);
        self.soloed.store(soloed, Ordering::Relaxed);
        self.connected.store(connected, Ordering::Relaxed);
    }
}

impl MasterControl {
    fn new(gain_db: f64, muted: bool) -> Self {
        Self {
            muted: AtomicBool::new(muted),
            gain: AtomicU32::new(db_to_gain(gain_db).to_bits()),
        }
    }

    pub fn update(&self, gain_db: f64, muted: bool) {
        self.gain
            .store(db_to_gain(gain_db).to_bits(), Ordering::Relaxed);
        self.muted.store(muted, Ordering::Relaxed);
    }
}

impl RenderPlan {
    pub fn build(project: &PlaybackProject, sample_rate: u32) -> Self {
        let mut duration_frames = 0;
        let mut track_controls = HashMap::new();
        let tracks = project
            .tracks
            .iter()
            .map(|track| {
                let control = Arc::new(TrackControl::new(
                    track.gain_db,
                    track.pan,
                    track.is_muted,
                    track.is_soloed,
                    track.is_connected,
                ));
                track_controls.insert(track.id.clone(), Arc::clone(&control));
                let clips = track
                    .clips
                    .iter()
                    .map(|clip| {
                        let start_frame = tick_to_frame(
                            clip.start_tick,
                            project.bpm,
                            project.ticks_per_quarter,
                            sample_rate,
                        );
                        let end_frame = tick_to_frame(
                            clip.start_tick.saturating_add(clip.duration_ticks),
                            project.bpm,
                            project.ticks_per_quarter,
                            sample_rate,
                        );
                        duration_frames = duration_frames.max(end_frame);
                        let timeline_duration_seconds =
                            frame_to_seconds(end_frame.saturating_sub(start_frame), sample_rate);
                        let source_duration_seconds = tick_to_seconds(
                            clip.source_duration_ticks,
                            project.bpm,
                            project.ticks_per_quarter,
                        );
                        let (left_gain, right_gain) = channel_gains(clip.gain_db, clip.pan);
                        RenderClip {
                            id: clip.id.clone(),
                            source_path: clip.source_path.clone(),
                            start_frame,
                            end_frame,
                            source_offset_seconds: tick_to_seconds(
                                clip.source_offset_ticks,
                                project.bpm,
                                project.ticks_per_quarter,
                            ),
                            source_duration_seconds,
                            source_rate: source_duration_seconds / timeline_duration_seconds,
                            stretch_mode: clip.stretch_mode,
                            pitch_ratio: 2.0_f64.powf(clip.pitch_semitones / 12.0),
                            left_gain,
                            right_gain,
                            fade_in_at_start: !track
                                .clips
                                .iter()
                                .any(|previous| clips_are_source_contiguous(previous, clip)),
                            fade_out_at_end: !track
                                .clips
                                .iter()
                                .any(|next| clips_are_source_contiguous(clip, next)),
                            fade_in_on_activation: false,
                        }
                    })
                    .collect();
                RenderTrack { control, clips }
            })
            .collect();
        let duration_ticks = frame_to_tick(
            duration_frames,
            project.bpm,
            project.ticks_per_quarter,
            sample_rate,
        );
        Self {
            sample_rate,
            bpm: project.bpm,
            ticks_per_quarter: project.ticks_per_quarter,
            duration_frames,
            duration_ticks,
            tracks,
            track_controls,
            master: Arc::new(MasterControl::new(
                project.master_gain_db,
                project.is_master_muted,
            )),
        }
    }

    pub fn mark_changed_clips(&mut self, previous: &Self) {
        for clip in self.tracks.iter_mut().flat_map(|track| &mut track.clips) {
            clip.fade_in_on_activation = !previous
                .tracks
                .iter()
                .flat_map(|track| &track.clips)
                .any(|previous| {
                    previous.id == clip.id
                        && previous.source_path == clip.source_path
                        && previous.start_frame == clip.start_frame
                        && previous.end_frame == clip.end_frame
                        && previous.source_offset_seconds.to_bits()
                            == clip.source_offset_seconds.to_bits()
                        && previous.source_duration_seconds.to_bits()
                            == clip.source_duration_seconds.to_bits()
                        && previous.stretch_mode == clip.stretch_mode
                        && previous.pitch_ratio.to_bits() == clip.pitch_ratio.to_bits()
                        && previous.left_gain.to_bits() == clip.left_gain.to_bits()
                        && previous.right_gain.to_bits() == clip.right_gain.to_bits()
                });
        }
    }
}

pub struct TimelineSource {
    state: PreparedRenderState,
    transport: Arc<TransportControl>,
    updates: Arc<ArrayQueue<PreparedRenderState>>,
    retired: Arc<ArrayQueue<PreparedRenderState>>,
    retirement_open: Arc<AtomicBool>,
    pending_retirement: Option<PreparedRenderState>,
    pending_update: Option<PreparedRenderState>,
    channel: u8,
    frame_samples: [f32; 2],
    advance_frame: bool,
}

#[derive(Default)]
pub struct AudioStreams {
    sources: HashMap<PathBuf, Arc<StreamedAudio>>,
}

pub struct PreparedRenderState {
    plan: Arc<RenderPlan>,
    streams: Vec<Vec<StreamCursor>>,
    stall_on_miss: bool,
    remap_position: bool,
    resume_if_at_end: bool,
    activation_frame: Option<u64>,
}

impl PreparedRenderState {
    /// Opens lightweight streaming decoders for each clip.
    ///
    /// # Errors
    ///
    /// Returns an error if a clip source cannot be opened or decoded.
    pub fn new(
        plan: Arc<RenderPlan>,
        audio_streams: &mut AudioStreams,
        stall_on_miss: bool,
        remap_position: bool,
        resume_if_at_end: bool,
    ) -> Result<Self, CoreError> {
        for clip in plan.tracks.iter().flat_map(|track| &track.clips) {
            if !audio_streams.sources.contains_key(&clip.source_path) {
                audio_streams.sources.insert(
                    clip.source_path.clone(),
                    Arc::new(StreamedAudio::open(&clip.source_path)?),
                );
            }
        }
        let streams = plan
            .tracks
            .iter()
            .map(|track| {
                track
                    .clips
                    .iter()
                    .map(|clip| {
                        audio_streams
                            .sources
                            .get(&clip.source_path)
                            .map(Arc::clone)
                            .map(StreamCursor::new)
                            .ok_or_else(|| CoreError::new("audio stream was not initialized"))
                    })
                    .collect()
            })
            .collect::<Result<Vec<_>, _>>()?;
        audio_streams.sources.retain(|path, _| {
            plan.tracks
                .iter()
                .flat_map(|track| &track.clips)
                .any(|clip| clip.source_path == *path)
        });
        Ok(Self {
            plan,
            streams,
            stall_on_miss,
            remap_position,
            resume_if_at_end,
            activation_frame: None,
        })
    }

    pub fn prewarm(&mut self, timeline_frame: u64) {
        let started = Instant::now();
        loop {
            if self.ready_at(timeline_frame) || started.elapsed() >= PREWARM_TIMEOUT {
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn ready_at(&mut self, timeline_frame: u64) -> bool {
        let mut ready = true;
        for (track, streams) in self.plan.tracks.iter().zip(&mut self.streams) {
            for (clip, stream) in track.clips.iter().zip(streams) {
                if timeline_frame >= clip.start_frame && timeline_frame < clip.end_frame {
                    ready &= clip_frames_ready(
                        clip,
                        stream,
                        timeline_frame - clip.start_frame,
                        self.plan.sample_rate,
                    );
                } else if clip.start_frame >= timeline_frame
                    && clip.start_frame - timeline_frame <= u64::from(self.plan.sample_rate)
                {
                    ready &= clip_frames_ready(clip, stream, 0, self.plan.sample_rate);
                }
            }
        }
        ready
    }
}

impl TimelineSource {
    /// Starts a permanent timeline source that accepts prepared plan updates.
    ///
    /// # Errors
    ///
    /// Returns an error if the retired-plan cleanup worker cannot be started.
    pub fn new(
        state: PreparedRenderState,
        transport: Arc<TransportControl>,
        updates: Arc<ArrayQueue<PreparedRenderState>>,
    ) -> Result<Self, CoreError> {
        let retired = Arc::new(ArrayQueue::new(RETIRED_PLAN_CAPACITY));
        let retirements = Arc::clone(&retired);
        let retirement_open = Arc::new(AtomicBool::new(true));
        let cleanup_open = Arc::clone(&retirement_open);
        thread::Builder::new()
            .name("audio-plan-cleanup".to_owned())
            .spawn(move || {
                while cleanup_open.load(Ordering::Acquire) || !retirements.is_empty() {
                    if let Some(state) = retirements.pop() {
                        drop(state);
                    } else {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            })
            .map_err(|error| {
                CoreError::new(format!(
                    "failed to start audio plan cleanup worker: {error}"
                ))
            })?;
        Ok(Self {
            state,
            transport,
            updates,
            retired,
            retirement_open,
            pending_retirement: None,
            pending_update: None,
            channel: 0,
            frame_samples: [0.0; 2],
            advance_frame: false,
        })
    }

    fn apply_pending_update(&mut self) {
        if let Some(retired) = self.pending_retirement.take()
            && let Err(retired) = self.retired.push(retired)
        {
            self.pending_retirement = Some(retired);
            return;
        }
        let mut next = if let Some(next) = self.pending_update.take() {
            next
        } else {
            let Some(next) = self.updates.pop() else {
                return;
            };
            next
        };
        let frame = self.transport.frame.load(Ordering::Acquire);
        let next_frame = if next.remap_position {
            tick_to_frame(
                frame_to_tick(
                    frame,
                    self.state.plan.bpm,
                    self.state.plan.ticks_per_quarter,
                    self.state.plan.sample_rate,
                ),
                next.plan.bpm,
                next.plan.ticks_per_quarter,
                next.plan.sample_rate,
            )
        } else {
            frame
        }
        .min(next.plan.duration_frames);
        if !next.ready_at(next_frame) {
            self.pending_update = Some(next);
            return;
        }
        if next.remap_position || frame > next.plan.duration_frames {
            self.transport.frame.store(next_frame, Ordering::Release);
        }
        if next.resume_if_at_end
            && PlaybackState::from_atomic(self.transport.state.load(Ordering::Acquire))
                == PlaybackState::Stopped
            && frame >= self.state.plan.duration_frames
        {
            self.transport
                .state
                .store(PlaybackState::Playing as u8, Ordering::Release);
        }
        next.activation_frame = Some(next_frame);
        let retired = std::mem::replace(&mut self.state, next);
        if let Err(retired) = self.retired.push(retired) {
            self.pending_retirement = Some(retired);
        }
    }

    fn render_frame(&mut self) {
        self.apply_pending_update();
        self.advance_frame = false;
        let requested = self
            .transport
            .requested_seek
            .swap(NO_SEEK, Ordering::AcqRel);
        if requested != NO_SEEK {
            self.transport.frame.store(
                requested.min(self.state.plan.duration_frames),
                Ordering::Release,
            );
        }
        if PlaybackState::from_atomic(self.transport.state.load(Ordering::Acquire))
            != PlaybackState::Playing
        {
            self.frame_samples = [0.0; 2];
            return;
        }
        let frame = self.transport.frame.load(Ordering::Acquire);
        if frame >= self.state.plan.duration_frames {
            self.transport
                .state
                .store(PlaybackState::Stopped as u8, Ordering::Release);
            self.frame_samples = [0.0; 2];
            return;
        }
        let has_solo = self
            .state
            .plan
            .tracks
            .iter()
            .any(|track| track.control.soloed.load(Ordering::Relaxed));
        let mut output = [0.0_f32; 2];
        let mut frame_ready = true;
        for (track, streams) in self.state.plan.tracks.iter().zip(&mut self.state.streams) {
            let audible = !track.control.muted.load(Ordering::Relaxed)
                && track.control.connected.load(Ordering::Relaxed)
                && (!has_solo || track.control.soloed.load(Ordering::Relaxed));
            let left_gain = f32::from_bits(track.control.left_gain.load(Ordering::Relaxed));
            let right_gain = f32::from_bits(track.control.right_gain.load(Ordering::Relaxed));
            for (clip, stream) in track.clips.iter().zip(streams) {
                if frame < clip.start_frame {
                    if clip.start_frame - frame <= u64::from(self.state.plan.sample_rate) {
                        request_clip_frames(clip, stream, 0, self.state.plan.sample_rate);
                    }
                    continue;
                }
                if frame >= clip.end_frame {
                    continue;
                }
                if !audible {
                    request_clip_frames(
                        clip,
                        stream,
                        frame - clip.start_frame,
                        self.state.plan.sample_rate,
                    );
                    continue;
                }
                if let Some([left, right]) = sample_clip(
                    clip,
                    stream,
                    frame - clip.start_frame,
                    self.state.plan.sample_rate,
                ) {
                    let envelope = clip_envelope(
                        clip,
                        frame,
                        self.state.activation_frame,
                        self.state.plan.sample_rate,
                    );
                    output[0] += left * clip.left_gain * left_gain * envelope;
                    output[1] += right * clip.right_gain * right_gain * envelope;
                } else if self.state.stall_on_miss {
                    frame_ready = false;
                }
            }
        }
        if !frame_ready {
            self.frame_samples = [0.0; 2];
            return;
        }
        if self.state.plan.master.muted.load(Ordering::Relaxed) {
            output = [0.0; 2];
        } else {
            let gain = f32::from_bits(self.state.plan.master.gain.load(Ordering::Relaxed));
            output[0] = (output[0] * gain).clamp(-1.0, 1.0);
            output[1] = (output[1] * gain).clamp(-1.0, 1.0);
        }
        self.frame_samples = output;
        self.advance_frame = true;
    }
}

impl Drop for TimelineSource {
    fn drop(&mut self) {
        self.retirement_open.store(false, Ordering::Release);
    }
}

struct StreamedAudio {
    sample_rate: u32,
    cache: Arc<ArcSwap<AudioBlockCache>>,
    requests: Arc<ArrayQueue<u64>>,
    decoder_open: Arc<AtomicBool>,
}

struct StreamCursor {
    audio: Arc<StreamedAudio>,
    block_index: Option<u64>,
    block: Option<Arc<Vec<[f32; 2]>>>,
    pending_requests: [Option<PendingBlockRequest>; 2],
}

#[derive(Clone, Copy)]
struct PendingBlockRequest {
    block_index: u64,
    misses: u16,
}

impl StreamCursor {
    fn new(audio: Arc<StreamedAudio>) -> Self {
        Self {
            audio,
            block_index: None,
            block: None,
            pending_requests: [None; 2],
        }
    }

    fn sample_rate(&self) -> u32 {
        self.audio.sample_rate
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn sample_at(&mut self, source_frame: f64) -> Option<[f32; 2]> {
        if !source_frame.is_finite() || source_frame < 0.0 {
            return Some([0.0; 2]);
        }
        let target_frame = source_frame.floor() as u64;
        let block_index = target_frame / DECODE_BLOCK_FRAMES;
        let frame_index = usize::try_from(target_frame % DECODE_BLOCK_FRAMES)
            .expect("decode block frame index fits in usize");
        if !self.load_block(block_index) {
            return None;
        }
        let block = self.block.as_ref().expect("loaded block should be present");
        if frame_index >= block.len() {
            return Some([0.0; 2]);
        }
        let lower = block[frame_index];
        let block_len = block.len();
        let upper = if frame_index + 1 < block_len {
            block[frame_index + 1]
        } else {
            if !self.load_block(block_index + 1) {
                return None;
            }
            self.block
                .as_ref()
                .and_then(|block| block.first())
                .copied()
                .unwrap_or([0.0; 2])
        };
        self.request_block(block_index + 1);
        let fraction = (source_frame - target_frame as f64) as f32;
        Some([
            lower[0] + (upper[0] - lower[0]) * fraction,
            lower[1] + (upper[1] - lower[1]) * fraction,
        ])
    }

    fn load_block(&mut self, block_index: u64) -> bool {
        if self.block_index == Some(block_index) {
            return true;
        }
        let Some(block) = self.audio.cached_block(block_index) else {
            if let Some(position) = self.pending_request_position(block_index) {
                let pending = self.pending_requests[position]
                    .as_mut()
                    .expect("pending request position should be occupied");
                pending.misses = pending.misses.saturating_add(1);
                if pending.misses < 256 {
                    return false;
                }
                self.pending_requests[position] = None;
            }
            self.request_block(block_index);
            return false;
        };
        if let Some(position) = self.pending_request_position(block_index) {
            self.pending_requests[position] = None;
        }
        self.block_index = Some(block_index);
        self.block = Some(block);
        true
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn load_frame(&mut self, source_frame: f64) -> bool {
        if !source_frame.is_finite() || source_frame < 0.0 {
            return true;
        }
        let frame = source_frame.floor() as u64;
        let block_index = frame / DECODE_BLOCK_FRAMES;
        self.load_block(block_index)
            && (frame % DECODE_BLOCK_FRAMES != DECODE_BLOCK_FRAMES - 1
                || self.load_block(block_index + 1))
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn request_frame(&mut self, source_frame: f64) {
        if source_frame.is_finite() && source_frame >= 0.0 {
            self.request_block(source_frame.floor() as u64 / DECODE_BLOCK_FRAMES);
        }
    }

    fn request_block(&mut self, block_index: u64) {
        if self.block_index == Some(block_index)
            || self.pending_request_position(block_index).is_some()
        {
            return;
        }
        let position =
            if let Some(position) = self.pending_requests.iter().position(Option::is_none) {
                position
            } else {
                self.clear_completed_requests();
                let Some(position) = self.pending_requests.iter().position(Option::is_none) else {
                    return;
                };
                position
            };
        if self.audio.request_block(block_index) {
            self.pending_requests[position] = Some(PendingBlockRequest {
                block_index,
                misses: 0,
            });
        }
    }

    fn pending_request_position(&self, block_index: u64) -> Option<usize> {
        self.pending_requests
            .iter()
            .position(|pending| pending.is_some_and(|pending| pending.block_index == block_index))
    }

    fn clear_completed_requests(&mut self) {
        for pending in &mut self.pending_requests {
            if pending
                .as_ref()
                .is_some_and(|pending| self.audio.cached_block(pending.block_index).is_some())
            {
                *pending = None;
            }
        }
    }
}

impl StreamedAudio {
    fn open(path: &Path) -> Result<Self, CoreError> {
        let file = File::open(path).map_err(|error| {
            CoreError::new(format!(
                "failed to open audio file {}: {error}",
                path.display()
            ))
        })?;
        let decoder = Decoder::try_from(file).map_err(|error| {
            CoreError::new(format!(
                "failed to decode audio file {}: {error}",
                path.display()
            ))
        })?;
        let channels = usize::from(decoder.channels());
        let sample_rate = decoder.sample_rate();
        if !(1..=2).contains(&channels) || sample_rate == 0 {
            return Err(CoreError::new(format!(
                "audio file must be non-empty mono or stereo: {}",
                path.display()
            )));
        }
        let cache = Arc::new(ArcSwap::from_pointee(AudioBlockCache::default()));
        let requests = Arc::new(ArrayQueue::new(DECODE_REQUEST_CAPACITY));
        let worker_requests = Arc::clone(&requests);
        let decoder_open = Arc::new(AtomicBool::new(true));
        let worker_open = Arc::clone(&decoder_open);
        let worker_cache = Arc::clone(&cache);
        let thread_name = format!(
            "decode-{}",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("audio")
        );
        thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                decode_worker(
                    decoder,
                    channels,
                    sample_rate,
                    &worker_cache,
                    &worker_requests,
                    &worker_open,
                );
            })
            .map_err(|error| {
                CoreError::new(format!(
                    "failed to start audio decoder for {}: {error}",
                    path.display()
                ))
            })?;
        Ok(Self {
            sample_rate,
            cache,
            requests,
            decoder_open,
        })
    }

    fn cached_block(&self, block_index: u64) -> Option<Arc<Vec<[f32; 2]>>> {
        self.cache.load().blocks.get(&block_index).cloned()
    }

    fn request_block(&self, block_index: u64) -> bool {
        self.requests.push(block_index).is_ok()
    }
}

impl Drop for StreamedAudio {
    fn drop(&mut self) {
        self.decoder_open.store(false, Ordering::Release);
    }
}

#[derive(Clone, Default)]
struct AudioBlockCache {
    blocks: HashMap<u64, Arc<Vec<[f32; 2]>>>,
    insertion_order: VecDeque<u64>,
}

impl AudioBlockCache {
    fn insert(&mut self, index: u64, frames: Vec<[f32; 2]>) {
        if self.blocks.contains_key(&index) {
            return;
        }
        while self.blocks.len() >= DECODE_CACHE_BLOCKS {
            if let Some(expired) = self.insertion_order.pop_front() {
                self.blocks.remove(&expired);
            }
        }
        self.blocks.insert(index, Arc::new(frames));
        self.insertion_order.push_back(index);
    }
}

fn decode_worker(
    mut decoder: Decoder<BufReader<File>>,
    channels: usize,
    sample_rate: u32,
    cache: &ArcSwap<AudioBlockCache>,
    requests: &ArrayQueue<u64>,
    decoder_open: &AtomicBool,
) {
    let mut decoder_frame = 0;
    while decoder_open.load(Ordering::Acquire) || !requests.is_empty() {
        let Some(requested_block) = requests.pop() else {
            thread::sleep(Duration::from_millis(1));
            continue;
        };
        for block_index in requested_block..requested_block.saturating_add(DECODE_PREFETCH_BLOCKS) {
            if cache_contains(cache, block_index) {
                continue;
            }
            let block_start = block_index.saturating_mul(DECODE_BLOCK_FRAMES);
            if decoder_frame != block_start {
                if decoder
                    .try_seek(frame_duration(block_start, sample_rate))
                    .is_err()
                {
                    insert_block(cache, block_index, Vec::new());
                    break;
                }
                decoder_frame = block_start;
            }
            let frames = decode_block(&mut decoder, channels);
            decoder_frame = decoder_frame.saturating_add(
                u64::try_from(frames.len()).expect("decoded frame count fits in u64"),
            );
            let reached_end = frames.len()
                < usize::try_from(DECODE_BLOCK_FRAMES).expect("decode block size fits in usize");
            insert_block(cache, block_index, frames);
            if reached_end {
                break;
            }
        }
    }
}

fn decode_block(decoder: &mut Decoder<BufReader<File>>, channels: usize) -> Vec<[f32; 2]> {
    let mut frames = Vec::with_capacity(
        usize::try_from(DECODE_BLOCK_FRAMES).expect("decode block size fits in usize"),
    );
    for _ in 0..DECODE_BLOCK_FRAMES {
        let Some(left) = decoder.next() else {
            break;
        };
        frames.push(if channels == 1 {
            [left, left]
        } else {
            [left, decoder.next().unwrap_or(left)]
        });
    }
    frames
}

fn cache_contains(cache: &ArcSwap<AudioBlockCache>, index: u64) -> bool {
    cache.load().blocks.contains_key(&index)
}

fn insert_block(cache: &ArcSwap<AudioBlockCache>, index: u64, frames: Vec<[f32; 2]>) {
    let mut updated = (**cache.load()).clone();
    updated.insert(index, frames);
    cache.store(Arc::new(updated));
}

impl Iterator for TimelineSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel == 0 {
            self.render_frame();
        }
        let sample = self.frame_samples[usize::from(self.channel)];
        self.channel = (self.channel + 1) % 2;
        if self.channel == 0
            && self.advance_frame
            && PlaybackState::from_atomic(self.transport.state.load(Ordering::Acquire))
                == PlaybackState::Playing
        {
            self.transport.frame.fetch_add(1, Ordering::AcqRel);
        }
        Some(sample)
    }
}

impl Source for TimelineSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.state.plan.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn tick_to_frame(tick: u32, bpm: f64, ticks_per_quarter: u32, sample_rate: u32) -> u64 {
    (tick_to_seconds(tick, bpm, ticks_per_quarter) * f64::from(sample_rate)).round() as u64
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn frame_to_tick(frame: u64, bpm: f64, ticks_per_quarter: u32, sample_rate: u32) -> u32 {
    (frame_to_seconds(frame, sample_rate) * bpm * f64::from(ticks_per_quarter) / 60.0)
        .round()
        .clamp(0.0, f64::from(u32::MAX)) as u32
}

pub fn tick_to_seconds(tick: u32, bpm: f64, ticks_per_quarter: u32) -> f64 {
    f64::from(tick) * 60.0 / (bpm * f64::from(ticks_per_quarter))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn seconds_to_ticks(seconds: f64, bpm: f64, ticks_per_quarter: u32) -> u32 {
    (seconds * bpm * f64::from(ticks_per_quarter) / 60.0)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32
}

fn clips_are_source_contiguous(left: &PlaybackClip, right: &PlaybackClip) -> bool {
    left.source_path == right.source_path
        && left.start_tick.checked_add(left.duration_ticks) == Some(right.start_tick)
        && left
            .source_offset_ticks
            .checked_add(left.source_duration_ticks)
            == Some(right.source_offset_ticks)
        && left.stretch_mode == right.stretch_mode
        && left.pitch_semitones.to_bits() == right.pitch_semitones.to_bits()
        && left.gain_db.to_bits() == right.gain_db.to_bits()
        && left.pan.to_bits() == right.pan.to_bits()
        && u64::from(left.source_duration_ticks) * u64::from(right.duration_ticks)
            == u64::from(right.source_duration_ticks) * u64::from(left.duration_ticks)
}

fn clip_source_seconds(clip: &RenderClip, local_frame: u64, sample_rate: u32) -> f64 {
    clip.source_offset_seconds + frame_to_seconds(local_frame, sample_rate) * clip.source_rate
}

fn sample_clip(
    clip: &RenderClip,
    stream: &mut StreamCursor,
    local_frame: u64,
    output_sample_rate: u32,
) -> Option<[f32; 2]> {
    if clip.stretch_mode == ClipStretchMode::Resample || clip_uses_unity_stretch(clip) {
        let source_seconds = clip_source_seconds(clip, local_frame, output_sample_rate);
        return sample_clip_at_seconds(clip, stream, source_seconds);
    }

    let mut mixed = [0.0_f32; 2];
    for (source_seconds, weight) in stretch_grains(clip, local_frame, output_sample_rate) {
        if weight <= f32::EPSILON {
            continue;
        }
        let sample = sample_clip_at_seconds(clip, stream, source_seconds)?;
        mixed[0] += sample[0] * weight;
        mixed[1] += sample[1] * weight;
    }
    Some(mixed)
}

fn clip_frames_ready(
    clip: &RenderClip,
    stream: &mut StreamCursor,
    local_frame: u64,
    output_sample_rate: u32,
) -> bool {
    let mut ready = true;
    for source_seconds in clip_source_frames(clip, local_frame, output_sample_rate) {
        ready &= stream.load_frame(source_seconds * f64::from(stream.sample_rate()));
    }
    ready
}

fn request_clip_frames(
    clip: &RenderClip,
    stream: &mut StreamCursor,
    local_frame: u64,
    output_sample_rate: u32,
) {
    for source_seconds in clip_source_frames(clip, local_frame, output_sample_rate) {
        stream.request_frame(source_seconds * f64::from(stream.sample_rate()));
    }
}

fn clip_source_frames(
    clip: &RenderClip,
    local_frame: u64,
    output_sample_rate: u32,
) -> impl Iterator<Item = f64> {
    let frames = if clip.stretch_mode == ClipStretchMode::Stretch && !clip_uses_unity_stretch(clip)
    {
        let grains = stretch_grains(clip, local_frame, output_sample_rate);
        [
            Some(grains[0].0),
            (grains[1].1 > f32::EPSILON).then_some(grains[1].0),
        ]
    } else {
        [
            Some(clip_source_seconds(clip, local_frame, output_sample_rate)),
            None,
        ]
    };
    frames.into_iter().flatten().filter(|source_seconds| {
        *source_seconds >= clip.source_offset_seconds
            && *source_seconds < clip.source_offset_seconds + clip.source_duration_seconds
    })
}

fn clip_uses_unity_stretch(clip: &RenderClip) -> bool {
    (clip.source_rate - 1.0).abs() < 1e-6 && (clip.pitch_ratio - 1.0).abs() < 1e-6
}

#[allow(clippy::cast_possible_truncation)]
fn stretch_grains(clip: &RenderClip, local_frame: u64, output_sample_rate: u32) -> [(f64, f32); 2] {
    let output_seconds = frame_to_seconds(local_frame, output_sample_rate);
    let grain_position = output_seconds / STRETCH_GRAIN_HOP_SECONDS;
    let grain_index = grain_position.floor();
    let phase = grain_position - grain_index;
    let blend = (phase * phase * (3.0 - 2.0 * phase)) as f32;
    let source_seconds = |index: f64| {
        let output_center = index * STRETCH_GRAIN_HOP_SECONDS;
        let source_center = clip.source_offset_seconds + output_center * clip.source_rate;
        source_center + (output_seconds - output_center) * clip.pitch_ratio
    };
    [
        (source_seconds(grain_index), 1.0 - blend),
        (source_seconds(grain_index + 1.0), blend),
    ]
}

fn sample_clip_at_seconds(
    clip: &RenderClip,
    stream: &mut StreamCursor,
    source_seconds: f64,
) -> Option<[f32; 2]> {
    if source_seconds < clip.source_offset_seconds
        || source_seconds >= clip.source_offset_seconds + clip.source_duration_seconds
    {
        return Some([0.0; 2]);
    }
    stream.sample_at(source_seconds * f64::from(stream.sample_rate()))
}

fn frame_to_seconds(frame: u64, sample_rate: u32) -> f64 {
    let sample_rate_u64 = u64::from(sample_rate);
    Duration::from_secs(frame / sample_rate_u64).as_secs_f64()
        + f64::from(
            u32::try_from(frame % sample_rate_u64)
                .expect("frame remainder is smaller than the sample rate"),
        ) / f64::from(sample_rate)
}

fn frame_duration(frame: u64, sample_rate: u32) -> Duration {
    let sample_rate = u64::from(sample_rate);
    let seconds = frame / sample_rate;
    let nanos = frame % sample_rate * 1_000_000_000 / sample_rate;
    Duration::from_secs(seconds) + Duration::from_nanos(nanos)
}

#[allow(clippy::cast_precision_loss)]
fn clip_envelope(
    clip: &RenderClip,
    frame: u64,
    activation_frame: Option<u64>,
    sample_rate: u32,
) -> f32 {
    let fade_frames = (u64::from(sample_rate) / 200).max(1);
    let natural_fade_in = if clip.fade_in_at_start {
        frame.saturating_sub(clip.start_frame).min(fade_frames)
    } else {
        fade_frames
    };
    let activation_fade_in = if clip.fade_in_on_activation {
        activation_frame.map_or(fade_frames, |activation| {
            frame.saturating_sub(activation).min(fade_frames)
        })
    } else {
        fade_frames
    };
    let fade_out = if clip.fade_out_at_end {
        clip.end_frame
            .saturating_sub(frame.saturating_add(1))
            .min(fade_frames)
    } else {
        fade_frames
    };
    natural_fade_in.min(activation_fade_in).min(fade_out) as f32 / fade_frames as f32
}

#[allow(clippy::cast_possible_truncation)]
fn db_to_gain(db: f64) -> f32 {
    10.0_f64.powf(db / 20.0) as f32
}

#[allow(clippy::cast_possible_truncation)]
fn channel_gains(db: f64, pan: f64) -> (f32, f32) {
    let gain = db_to_gain(db);
    (
        gain * (1.0 - pan.max(0.0)) as f32,
        gain * (1.0 + pan.min(0.0)) as f32,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicU8, AtomicU64, Ordering},
    };

    use crossbeam_queue::ArrayQueue;

    use crate::workspace::{ClipStretchMode, PlaybackClip, PlaybackProject, PlaybackTrack};

    use super::{
        AudioStreams, NO_SEEK, PlaybackState, PreparedRenderState, RenderPlan, TimelineSource,
        TransportControl, clip_envelope, frame_to_tick, seconds_to_ticks, tick_to_frame,
    };

    #[test]
    fn converts_between_ticks_and_frames() {
        assert_eq!(tick_to_frame(960, 120.0, 960, 48_000), 24_000);
        assert_eq!(frame_to_tick(24_000, 120.0, 960, 48_000), 960);
        assert_eq!(seconds_to_ticks(0.5, 120.0, 960), 960);
    }

    #[test]
    fn contiguous_clips_from_one_source_do_not_fade_at_the_split() {
        let project = PlaybackProject {
            bpm: 120.0,
            ticks_per_quarter: 960,
            master_gain_db: 0.0,
            is_master_muted: false,
            tracks: vec![PlaybackTrack {
                id: "track-1".to_owned(),
                is_muted: false,
                is_soloed: false,
                gain_db: 0.0,
                pan: 0.0,
                is_connected: true,
                clips: vec![
                    PlaybackClip {
                        id: "clip-1".to_owned(),
                        source_path: "Kick.wav".into(),
                        start_tick: 0,
                        duration_ticks: 960,
                        source_offset_ticks: 0,
                        source_duration_ticks: 960,
                        stretch_mode: ClipStretchMode::Resample,
                        gain_db: 0.0,
                        pan: 0.0,
                        pitch_semitones: 0.0,
                    },
                    PlaybackClip {
                        id: "clip-2".to_owned(),
                        source_path: "Kick.wav".into(),
                        start_tick: 960,
                        duration_ticks: 960,
                        source_offset_ticks: 960,
                        source_duration_ticks: 960,
                        stretch_mode: ClipStretchMode::Resample,
                        gain_db: 0.0,
                        pan: 0.0,
                        pitch_semitones: 0.0,
                    },
                ],
            }],
        };

        let plan = RenderPlan::build(&project, 48_000);
        let left = &plan.tracks[0].clips[0];
        let right = &plan.tracks[0].clips[1];

        assert!(!left.fade_out_at_end);
        assert!(!right.fade_in_at_start);
        assert!((clip_envelope(left, left.end_frame - 1, None, 48_000) - 1.0).abs() < f32::EPSILON);
        assert!((clip_envelope(right, right.start_frame, None, 48_000) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn render_plan_updates_apply_between_stereo_frames() {
        let project = PlaybackProject {
            bpm: 120.0,
            ticks_per_quarter: 960,
            master_gain_db: 0.0,
            is_master_muted: false,
            tracks: Vec::new(),
        };
        let plan = Arc::new(RenderPlan::build(&project, 48_000));
        let transport = Arc::new(TransportControl {
            state: AtomicU8::new(PlaybackState::Paused as u8),
            frame: AtomicU64::new(0),
            requested_seek: AtomicU64::new(NO_SEEK),
        });
        let mut streams = AudioStreams::default();
        let initial = PreparedRenderState::new(plan, &mut streams, false, false, false)
            .expect("empty source should build");
        let updates = Arc::new(ArrayQueue::new(1));
        let mut source = TimelineSource::new(initial, transport, Arc::clone(&updates))
            .expect("timeline source should build");
        assert_eq!(source.next(), Some(0.0));

        let updated_project = PlaybackProject {
            bpm: 130.0,
            ..project
        };
        let next_state = PreparedRenderState::new(
            Arc::new(RenderPlan::build(&updated_project, 48_000)),
            &mut streams,
            false,
            true,
            false,
        )
        .expect("updated source should build");
        assert!(updates.push(next_state).is_ok());

        assert_eq!(source.next(), Some(0.0));
        assert_eq!(source.state.plan.bpm.to_bits(), 120.0_f64.to_bits());
        assert_eq!(source.next(), Some(0.0));
        assert_eq!(source.state.plan.bpm.to_bits(), 130.0_f64.to_bits());
    }

    #[test]
    fn render_plan_updates_preserve_playback_state_and_frame() {
        let project = PlaybackProject {
            bpm: 120.0,
            ticks_per_quarter: 960,
            master_gain_db: 0.0,
            is_master_muted: false,
            tracks: Vec::new(),
        };
        let mut initial_plan = RenderPlan::build(&project, 48_000);
        initial_plan.duration_frames = 1_000;
        initial_plan.duration_ticks = frame_to_tick(1_000, 120.0, 960, 48_000);
        let transport = Arc::new(TransportControl {
            state: AtomicU8::new(PlaybackState::Playing as u8),
            frame: AtomicU64::new(100),
            requested_seek: AtomicU64::new(NO_SEEK),
        });
        let mut streams = AudioStreams::default();
        let initial =
            PreparedRenderState::new(Arc::new(initial_plan), &mut streams, false, false, false)
                .expect("initial source should build");
        let updates = Arc::new(ArrayQueue::new(1));
        let mut source = TimelineSource::new(initial, Arc::clone(&transport), Arc::clone(&updates))
            .expect("timeline source should build");

        let mut next_plan = RenderPlan::build(&project, 48_000);
        next_plan.duration_frames = 2_000;
        next_plan.duration_ticks = frame_to_tick(2_000, 120.0, 960, 48_000);
        let next_state =
            PreparedRenderState::new(Arc::new(next_plan), &mut streams, false, false, false)
                .expect("updated source should build");
        assert!(updates.push(next_state).is_ok());

        assert_eq!(source.next(), Some(0.0));
        assert_eq!(source.next(), Some(0.0));
        assert_eq!(transport.frame.load(Ordering::Acquire), 101);
        assert_eq!(
            PlaybackState::from_atomic(transport.state.load(Ordering::Acquire)),
            PlaybackState::Playing
        );
    }
}
