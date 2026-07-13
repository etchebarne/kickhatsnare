use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering},
    },
    time::Duration,
};

use rodio::Source;

use super::decoder::DecodedAudio;
use crate::workspace::PlaybackProject;

pub const NO_SEEK: u64 = u64::MAX;

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
    audio: Arc<DecodedAudio>,
    start_frame: u64,
    end_frame: u64,
    source_offset_seconds: f64,
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
    pub fn build(
        project: &PlaybackProject,
        sample_rate: u32,
        decoded: &HashMap<std::path::PathBuf, Arc<DecodedAudio>>,
    ) -> Self {
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
                    .filter_map(|clip| {
                        let audio = Arc::clone(decoded.get(&clip.source_path)?);
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
                        Some(RenderClip {
                            audio,
                            start_frame,
                            end_frame,
                            source_offset_seconds: tick_to_seconds(
                                clip.source_offset_ticks,
                                project.bpm,
                                project.ticks_per_quarter,
                            ),
                        })
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
}

pub struct TimelineSource {
    plan: Arc<RenderPlan>,
    transport: Arc<TransportControl>,
    channel: u8,
    frame_samples: [f32; 2],
}

impl TimelineSource {
    pub fn new(plan: Arc<RenderPlan>, transport: Arc<TransportControl>) -> Self {
        Self {
            plan,
            transport,
            channel: 0,
            frame_samples: [0.0; 2],
        }
    }

    fn render_frame(&mut self) {
        let requested = self
            .transport
            .requested_seek
            .swap(NO_SEEK, Ordering::AcqRel);
        if requested != NO_SEEK {
            self.transport
                .frame
                .store(requested.min(self.plan.duration_frames), Ordering::Release);
        }
        if PlaybackState::from_atomic(self.transport.state.load(Ordering::Acquire))
            != PlaybackState::Playing
        {
            self.frame_samples = [0.0; 2];
            return;
        }
        let frame = self.transport.frame.load(Ordering::Acquire);
        if frame >= self.plan.duration_frames {
            self.transport
                .state
                .store(PlaybackState::Stopped as u8, Ordering::Release);
            self.frame_samples = [0.0; 2];
            return;
        }
        let has_solo = self
            .plan
            .tracks
            .iter()
            .any(|track| track.control.soloed.load(Ordering::Relaxed));
        let mut output = [0.0_f32; 2];
        for track in &self.plan.tracks {
            if track.control.muted.load(Ordering::Relaxed)
                || !track.control.connected.load(Ordering::Relaxed)
                || (has_solo && !track.control.soloed.load(Ordering::Relaxed))
            {
                continue;
            }
            let left_gain = f32::from_bits(track.control.left_gain.load(Ordering::Relaxed));
            let right_gain = f32::from_bits(track.control.right_gain.load(Ordering::Relaxed));
            for clip in &track.clips {
                if frame < clip.start_frame || frame >= clip.end_frame {
                    continue;
                }
                let seconds = frame_to_seconds(frame - clip.start_frame, self.plan.sample_rate)
                    + clip.source_offset_seconds;
                let source_frame = seconds * f64::from(clip.audio.sample_rate);
                let [left, right] = sample_stereo(&clip.audio, source_frame);
                output[0] += left * left_gain;
                output[1] += right * right_gain;
            }
        }
        if self.plan.master.muted.load(Ordering::Relaxed) {
            output = [0.0; 2];
        } else {
            let gain = f32::from_bits(self.plan.master.gain.load(Ordering::Relaxed));
            output[0] = (output[0] * gain).clamp(-1.0, 1.0);
            output[1] = (output[1] * gain).clamp(-1.0, 1.0);
        }
        self.frame_samples = output;
    }
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
        self.plan.sample_rate
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

fn frame_to_seconds(frame: u64, sample_rate: u32) -> f64 {
    let sample_rate_u64 = u64::from(sample_rate);
    Duration::from_secs(frame / sample_rate_u64).as_secs_f64()
        + f64::from(
            u32::try_from(frame % sample_rate_u64)
                .expect("frame remainder is smaller than the sample rate"),
        ) / f64::from(sample_rate)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn sample_stereo(audio: &DecodedAudio, source_frame: f64) -> [f32; 2] {
    if source_frame < 0.0 || source_frame >= audio.frame_count as f64 {
        return [0.0; 2];
    }
    let frame = source_frame.floor() as usize;
    let next = (frame + 1).min(audio.frame_count.saturating_sub(1));
    let fraction = (source_frame - frame as f64) as f32;
    let channels = usize::from(audio.channels);
    let sample = |frame: usize, channel: usize| {
        let channel = channel.min(channels - 1);
        audio.samples[frame * channels + channel]
    };
    let interpolate = |channel| {
        let first = sample(frame, channel);
        first + (sample(next, channel) - first) * fraction
    };
    if channels == 1 {
        let mono = interpolate(0);
        [mono, mono]
    } else {
        [interpolate(0), interpolate(1)]
    }
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
    use super::{frame_to_tick, seconds_to_ticks, tick_to_frame};

    #[test]
    fn converts_between_ticks_and_frames() {
        assert_eq!(tick_to_frame(960, 120.0, 960, 48_000), 24_000);
        assert_eq!(frame_to_tick(24_000, 120.0, 960, 48_000), 960);
        assert_eq!(seconds_to_ticks(0.5, 120.0, 960), 960);
    }
}
