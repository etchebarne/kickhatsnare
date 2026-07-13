use std::{collections::HashSet, path::Path};

use serde::{Deserialize, Serialize};

use crate::CoreError;

pub const TICKS_PER_QUARTER: u32 = 960;

const DEFAULT_BPM: f64 = 120.0;
const DEFAULT_TRACK_COUNT: u64 = 30;
const DEFAULT_TIME_SIGNATURE_NUMERATOR: u8 = 4;
const DEFAULT_TIME_SIGNATURE_DENOMINATOR: u8 = 4;
const MASTER_NODE_ID: &str = "master";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GridDivision {
    Whole,
    Half,
    #[default]
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    QuarterTriplet,
    EighthTriplet,
    SixteenthTriplet,
}

impl GridDivision {
    #[must_use]
    pub const fn ticks(self) -> u32 {
        match self {
            Self::Whole => TICKS_PER_QUARTER * 4,
            Self::Half => TICKS_PER_QUARTER * 2,
            Self::Quarter => TICKS_PER_QUARTER,
            Self::Eighth => TICKS_PER_QUARTER / 2,
            Self::Sixteenth => TICKS_PER_QUARTER / 4,
            Self::ThirtySecond => TICKS_PER_QUARTER / 8,
            Self::QuarterTriplet => TICKS_PER_QUARTER * 2 / 3,
            Self::EighthTriplet => TICKS_PER_QUARTER / 3,
            Self::SixteenthTriplet => TICKS_PER_QUARTER / 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSnapshot {
    pub ticks_per_quarter: u32,
    pub bpm: f64,
    pub time_signature_numerator: u8,
    pub time_signature_denominator: u8,
    pub grid_division: GridDivision,
    pub is_snap_enabled: bool,
    pub master_gain_db: f64,
    pub is_master_muted: bool,
    pub master_node_x: f64,
    pub master_node_y: f64,
    pub tracks: Vec<TimelineTrackSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineTrackSnapshot {
    pub id: String,
    pub name: String,
    pub is_muted: bool,
    pub is_soloed: bool,
    pub gain_db: f64,
    pub pan: f64,
    pub is_connected: bool,
    pub node_x: f64,
    pub node_y: f64,
    pub clips: Vec<TimelineClipSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClipSnapshot {
    pub id: String,
    pub name: String,
    pub start_tick: u32,
    pub duration_ticks: u32,
    pub source_offset_ticks: u32,
    pub source_path: Option<String>,
    pub source_sample_rate: u32,
    pub source_channels: u16,
    pub source_duration_seconds: f64,
    pub waveform: Vec<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Timeline {
    bpm: f64,
    time_signature_numerator: u8,
    time_signature_denominator: u8,
    grid_division: GridDivision,
    is_snap_enabled: bool,
    #[serde(default)]
    master_gain_db: f64,
    #[serde(default)]
    is_master_muted: bool,
    #[serde(default = "default_master_x")]
    master_node_x: f64,
    #[serde(default = "default_master_y")]
    master_node_y: f64,
    tracks: Vec<TimelineTrack>,
    next_track_id: u64,
    next_clip_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TimelineTrack {
    id: String,
    name: String,
    is_muted: bool,
    is_soloed: bool,
    #[serde(default)]
    gain_db: f64,
    #[serde(default)]
    pan: f64,
    #[serde(default = "default_true")]
    is_connected: bool,
    #[serde(default)]
    node_x: f64,
    #[serde(default)]
    node_y: f64,
    clips: Vec<TimelineClip>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TimelineClip {
    id: String,
    name: String,
    start_tick: u32,
    duration_ticks: u32,
    source_offset_ticks: u32,
    #[serde(default)]
    source_path: Option<String>,
    #[serde(default)]
    source_sample_rate: u32,
    #[serde(default)]
    source_channels: u16,
    #[serde(default)]
    source_duration_seconds: f64,
    #[serde(default)]
    waveform: Vec<f32>,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            bpm: DEFAULT_BPM,
            time_signature_numerator: DEFAULT_TIME_SIGNATURE_NUMERATOR,
            time_signature_denominator: DEFAULT_TIME_SIGNATURE_DENOMINATOR,
            grid_division: GridDivision::Quarter,
            is_snap_enabled: true,
            master_gain_db: 0.0,
            is_master_muted: false,
            master_node_x: default_master_x(),
            master_node_y: default_master_y(),
            tracks: (1..=DEFAULT_TRACK_COUNT).map(default_track).collect(),
            next_track_id: DEFAULT_TRACK_COUNT + 1,
            next_clip_id: 1,
        }
    }
}

impl Timeline {
    pub(super) fn snapshot(&self) -> TimelineSnapshot {
        TimelineSnapshot {
            ticks_per_quarter: TICKS_PER_QUARTER,
            bpm: self.bpm,
            time_signature_numerator: self.time_signature_numerator,
            time_signature_denominator: self.time_signature_denominator,
            grid_division: self.grid_division,
            is_snap_enabled: self.is_snap_enabled,
            master_gain_db: self.master_gain_db,
            is_master_muted: self.is_master_muted,
            master_node_x: self.master_node_x,
            master_node_y: self.master_node_y,
            tracks: self
                .tracks
                .iter()
                .map(|track| TimelineTrackSnapshot {
                    id: track.id.clone(),
                    name: track.name.clone(),
                    is_muted: track.is_muted,
                    is_soloed: track.is_soloed,
                    gain_db: track.gain_db,
                    pan: track.pan,
                    is_connected: track.is_connected,
                    node_x: track.node_x,
                    node_y: track.node_y,
                    clips: track
                        .clips
                        .iter()
                        .map(|clip| TimelineClipSnapshot {
                            id: clip.id.clone(),
                            name: clip.name.clone(),
                            start_tick: clip.start_tick,
                            duration_ticks: clip.duration_ticks,
                            source_offset_ticks: clip.source_offset_ticks,
                            source_path: clip.source_path.clone(),
                            source_sample_rate: clip.source_sample_rate,
                            source_channels: clip.source_channels,
                            source_duration_seconds: clip.source_duration_seconds,
                            waveform: clip.waveform.clone(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    pub(super) fn validate(&self) -> Result<(), CoreError> {
        validate_settings(
            self.bpm,
            self.time_signature_numerator,
            self.time_signature_denominator,
        )?;
        validate_gain(self.master_gain_db, "master")?;
        validate_position(self.master_node_x, self.master_node_y)?;
        let mut track_ids = HashSet::new();
        let mut clip_ids = HashSet::new();
        for track in &self.tracks {
            validate_name(&track.name, "track")?;
            validate_gain(track.gain_db, "track")?;
            validate_pan(track.pan)?;
            validate_position(track.node_x, track.node_y)?;
            if track.id.is_empty() || !track_ids.insert(track.id.as_str()) {
                return Err(CoreError::new(
                    "project contains an invalid or duplicate track ID",
                ));
            }
            for clip in &track.clips {
                validate_name(&clip.name, "clip")?;
                validate_clip_range(clip.start_tick, clip.duration_ticks)?;
                validate_source(clip, self.bpm)?;
                if clip.id.is_empty() || !clip_ids.insert(clip.id.as_str()) {
                    return Err(CoreError::new(
                        "project contains an invalid or duplicate clip ID",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(super) fn ensure_minimum_track(&mut self) -> Result<(), CoreError> {
        if self.tracks.is_empty() {
            let id = self.next_track_id()?;
            self.tracks.push(TimelineTrack {
                id,
                name: "Track 1".to_owned(),
                is_muted: false,
                is_soloed: false,
                gain_db: 0.0,
                pan: 0.0,
                is_connected: true,
                node_x: 0.0,
                node_y: 0.0,
                clips: Vec::new(),
            });
        }
        Ok(())
    }

    pub(super) fn migrate_from(&mut self, format_version: u32) {
        if format_version < 3 {
            self.master_node_x = default_master_x();
            self.master_node_y = default_master_y();
            for (index, track) in self.tracks.iter_mut().enumerate() {
                (track.node_x, track.node_y) = default_track_position(index as u64 + 1);
            }
        }
    }

    pub(super) fn set_settings(
        &mut self,
        bpm: f64,
        time_signature_numerator: u8,
        time_signature_denominator: u8,
        grid_division: GridDivision,
        is_snap_enabled: bool,
    ) -> Result<(), CoreError> {
        validate_settings(bpm, time_signature_numerator, time_signature_denominator)?;
        let mut updated = self.clone();
        if (bpm - self.bpm).abs() > f64::EPSILON {
            let scale = bpm / self.bpm;
            for clip in updated
                .tracks
                .iter_mut()
                .flat_map(|track| &mut track.clips)
                .filter(|clip| clip.source_path.is_some())
            {
                clip.source_offset_ticks = scale_ticks(clip.source_offset_ticks, scale)?;
                clip.duration_ticks = scale_ticks(clip.duration_ticks, scale)?.max(1);

                let source_ticks = seconds_to_ticks(clip.source_duration_seconds, bpm).max(1);
                if clip.source_offset_ticks >= source_ticks {
                    return Err(CoreError::new(
                        "audio clip trim exceeds the source duration",
                    ));
                }
                clip.duration_ticks = clip
                    .duration_ticks
                    .min(source_ticks - clip.source_offset_ticks);
                validate_clip_range(clip.start_tick, clip.duration_ticks)?;
            }
        }
        updated.bpm = bpm;
        updated.time_signature_numerator = time_signature_numerator;
        updated.time_signature_denominator = time_signature_denominator;
        updated.grid_division = grid_division;
        updated.is_snap_enabled = is_snap_enabled;
        *self = updated;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn save_track(
        &mut self,
        id: Option<&str>,
        name: &str,
        is_muted: bool,
        is_soloed: bool,
        gain_db: f64,
        pan: f64,
        is_connected: bool,
    ) -> Result<(), CoreError> {
        let name = validate_name(name, "track")?;
        validate_gain(gain_db, "track")?;
        validate_pan(pan)?;
        if let Some(id) = id {
            let track = self
                .tracks
                .iter_mut()
                .find(|track| track.id == id)
                .ok_or_else(|| CoreError::new("timeline track does not exist"))?;
            track.name = name;
            track.is_muted = is_muted;
            track.is_soloed = is_soloed;
            track.gain_db = gain_db;
            track.pan = pan;
            track.is_connected = is_connected;
        } else {
            let id = self.next_track_id()?;
            let number = id
                .strip_prefix("track-")
                .and_then(|number| number.parse().ok())
                .unwrap_or(self.tracks.len() as u64 + 1);
            let (node_x, node_y) = default_track_position(number);
            self.tracks.push(TimelineTrack {
                id,
                name,
                is_muted,
                is_soloed,
                gain_db,
                pan,
                is_connected,
                node_x,
                node_y,
                clips: Vec::new(),
            });
        }
        Ok(())
    }

    pub(super) fn delete_track(&mut self, id: &str) -> Result<(), CoreError> {
        let index = self
            .tracks
            .iter()
            .position(|track| track.id == id)
            .ok_or_else(|| CoreError::new("timeline track does not exist"))?;
        if self.tracks.len() == 1 {
            return Err(CoreError::new(
                "a project must contain at least one timeline track",
            ));
        }
        self.tracks.remove(index);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn save_clip(
        &mut self,
        id: Option<&str>,
        track_id: &str,
        name: &str,
        start_tick: u32,
        duration_ticks: u32,
        source_offset_ticks: u32,
    ) -> Result<(), CoreError> {
        let name = validate_name(name, "clip")?;
        validate_clip_range(start_tick, duration_ticks)?;
        let target_track_index = self
            .tracks
            .iter()
            .position(|track| track.id == track_id)
            .ok_or_else(|| CoreError::new("timeline track does not exist"))?;

        if let Some(id) = id {
            let (source_track_index, clip_index) = self
                .tracks
                .iter()
                .enumerate()
                .find_map(|(track_index, track)| {
                    track
                        .clips
                        .iter()
                        .position(|clip| clip.id == id)
                        .map(|clip_index| (track_index, clip_index))
                })
                .ok_or_else(|| CoreError::new("timeline clip does not exist"))?;
            let existing = &self.tracks[source_track_index].clips[clip_index];
            if existing.source_path.is_some() {
                let source_ticks = seconds_to_ticks(existing.source_duration_seconds, self.bpm);
                if source_offset_ticks.saturating_add(duration_ticks) > source_ticks {
                    return Err(CoreError::new(
                        "audio clip trim exceeds the source duration",
                    ));
                }
            }
            let mut clip = self.tracks[source_track_index].clips.remove(clip_index);
            clip.name = name;
            clip.start_tick = start_tick;
            clip.duration_ticks = duration_ticks;
            clip.source_offset_ticks = source_offset_ticks;
            self.tracks[target_track_index].clips.push(clip);
        } else {
            let id = self.next_clip_id()?;
            self.tracks[target_track_index].clips.push(TimelineClip {
                id,
                name,
                start_tick,
                duration_ticks,
                source_offset_ticks,
                source_path: None,
                source_sample_rate: 0,
                source_channels: 0,
                source_duration_seconds: 0.0,
                waveform: Vec::new(),
            });
        }
        self.tracks[target_track_index]
            .clips
            .sort_by_key(|clip| (clip.start_tick, clip.id.clone()));
        Ok(())
    }

    pub(super) fn delete_clip(&mut self, id: &str) -> Result<(), CoreError> {
        for track in &mut self.tracks {
            if let Some(index) = track.clips.iter().position(|clip| clip.id == id) {
                track.clips.remove(index);
                return Ok(());
            }
        }
        Err(CoreError::new("timeline clip does not exist"))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn add_audio_clip(
        &mut self,
        track_id: &str,
        source_path: &str,
        name: &str,
        start_tick: u32,
        duration_ticks: u32,
        sample_rate: u32,
        channels: u16,
        duration_seconds: f64,
        waveform: Vec<f32>,
    ) -> Result<(), CoreError> {
        let target_track_index = self
            .tracks
            .iter()
            .position(|track| track.id == track_id)
            .ok_or_else(|| CoreError::new("timeline track does not exist"))?;
        let name = validate_name(name, "clip")?;
        validate_clip_range(start_tick, duration_ticks)?;
        let source_path = validate_source_path(source_path)?;
        if sample_rate == 0 || !(1..=2).contains(&channels) {
            return Err(CoreError::new("audio clip metadata is invalid"));
        }
        if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
            return Err(CoreError::new("audio clip duration is invalid"));
        }
        if waveform.len() > 2_048 || waveform.iter().any(|peak| !peak.is_finite()) {
            return Err(CoreError::new("audio clip waveform is invalid"));
        }
        let id = self.next_clip_id()?;
        self.tracks[target_track_index].clips.push(TimelineClip {
            id,
            name,
            start_tick,
            duration_ticks,
            source_offset_ticks: 0,
            source_path: Some(source_path),
            source_sample_rate: sample_rate,
            source_channels: channels,
            source_duration_seconds: duration_seconds,
            waveform,
        });
        self.tracks[target_track_index]
            .clips
            .sort_by_key(|clip| (clip.start_tick, clip.id.clone()));
        Ok(())
    }

    pub(super) fn set_master_mix(&mut self, gain_db: f64, is_muted: bool) -> Result<(), CoreError> {
        validate_gain(gain_db, "master")?;
        self.master_gain_db = gain_db;
        self.is_master_muted = is_muted;
        Ok(())
    }

    pub(super) fn set_node_position(
        &mut self,
        node_id: &str,
        x: f64,
        y: f64,
    ) -> Result<(), CoreError> {
        validate_position(x, y)?;
        if node_id == MASTER_NODE_ID {
            self.master_node_x = x;
            self.master_node_y = y;
            return Ok(());
        }
        let track = self
            .tracks
            .iter_mut()
            .find(|track| track.id == node_id)
            .ok_or_else(|| CoreError::new("mix node does not exist"))?;
        track.node_x = x;
        track.node_y = y;
        Ok(())
    }

    pub(super) fn source_is_referenced(&self, path: &Path) -> bool {
        self.tracks
            .iter()
            .flat_map(|track| &track.clips)
            .any(|clip| {
                clip.source_path
                    .as_deref()
                    .is_some_and(|source| Path::new(source).starts_with(path))
            })
    }

    pub(super) fn move_source_paths(&mut self, source: &Path, destination: &Path) -> bool {
        let mut changed = false;
        for clip in self.tracks.iter_mut().flat_map(|track| &mut track.clips) {
            let Some(path) = clip.source_path.as_deref().map(Path::new) else {
                continue;
            };
            if let Ok(suffix) = path.strip_prefix(source) {
                let moved_path = if suffix.as_os_str().is_empty() {
                    destination.to_owned()
                } else {
                    destination.join(suffix)
                };
                clip.source_path = Some(path_string(&moved_path));
                changed = true;
            }
        }
        changed
    }

    fn next_track_id(&mut self) -> Result<String, CoreError> {
        loop {
            let id = format!("track-{}", self.next_track_id);
            self.next_track_id = self
                .next_track_id
                .checked_add(1)
                .ok_or_else(|| CoreError::new("timeline track ID space is exhausted"))?;
            if self.tracks.iter().all(|track| track.id != id) {
                return Ok(id);
            }
        }
    }

    fn next_clip_id(&mut self) -> Result<String, CoreError> {
        loop {
            let id = format!("clip-{}", self.next_clip_id);
            self.next_clip_id = self
                .next_clip_id
                .checked_add(1)
                .ok_or_else(|| CoreError::new("timeline clip ID space is exhausted"))?;
            if self
                .tracks
                .iter()
                .flat_map(|track| &track.clips)
                .all(|clip| clip.id != id)
            {
                return Ok(id);
            }
        }
    }
}

fn default_track(number: u64) -> TimelineTrack {
    let (node_x, node_y) = default_track_position(number);
    TimelineTrack {
        id: format!("track-{number}"),
        name: format!("Track {number}"),
        is_muted: false,
        is_soloed: false,
        gain_db: 0.0,
        pan: 0.0,
        is_connected: true,
        node_x,
        node_y,
        clips: Vec::new(),
    }
}

fn default_track_position(number: u64) -> (f64, f64) {
    let index = number.saturating_sub(1);
    let column = u32::try_from(index % 5).expect("track column is always less than five");
    let row = u32::try_from(index / 5).unwrap_or(u32::MAX);
    (f64::from(column) * 280.0, f64::from(row) * 180.0)
}

fn default_master_x() -> f64 {
    1_600.0
}

fn default_master_y() -> f64 {
    420.0
}

fn default_true() -> bool {
    true
}

fn validate_settings(
    bpm: f64,
    time_signature_numerator: u8,
    time_signature_denominator: u8,
) -> Result<(), CoreError> {
    if !bpm.is_finite() || !(20.0..=400.0).contains(&bpm) {
        return Err(CoreError::new("timeline BPM must be between 20 and 400"));
    }
    if !(1..=32).contains(&time_signature_numerator) {
        return Err(CoreError::new(
            "time signature numerator must be between 1 and 32",
        ));
    }
    if !matches!(time_signature_denominator, 1 | 2 | 4 | 8 | 16 | 32) {
        return Err(CoreError::new(
            "time signature denominator must be 1, 2, 4, 8, 16, or 32",
        ));
    }
    Ok(())
}

fn validate_name(name: &str, entity: &str) -> Result<String, CoreError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::new(format!(
            "timeline {entity} name cannot be empty"
        )));
    }
    if name.chars().count() > 100 {
        return Err(CoreError::new(format!(
            "timeline {entity} name cannot exceed 100 characters"
        )));
    }
    Ok(name.to_owned())
}

fn validate_gain(gain_db: f64, entity: &str) -> Result<(), CoreError> {
    if !gain_db.is_finite() || !(-60.0..=12.0).contains(&gain_db) {
        return Err(CoreError::new(format!(
            "{entity} gain must be between -60 and 12 dB"
        )));
    }
    Ok(())
}

fn validate_pan(pan: f64) -> Result<(), CoreError> {
    if !pan.is_finite() || !(-1.0..=1.0).contains(&pan) {
        return Err(CoreError::new("track pan must be between -1 and 1"));
    }
    Ok(())
}

fn validate_position(x: f64, y: f64) -> Result<(), CoreError> {
    if !x.is_finite() || !y.is_finite() || x.abs() > 100_000.0 || y.abs() > 100_000.0 {
        return Err(CoreError::new("mix node position is invalid"));
    }
    Ok(())
}

fn validate_source(clip: &TimelineClip, bpm: f64) -> Result<(), CoreError> {
    let Some(source_path) = clip.source_path.as_deref() else {
        return Ok(());
    };
    validate_source_path(source_path)?;
    if clip.source_sample_rate == 0
        || !(1..=2).contains(&clip.source_channels)
        || !clip.source_duration_seconds.is_finite()
        || clip.source_duration_seconds <= 0.0
    {
        return Err(CoreError::new("audio clip metadata is invalid"));
    }
    if clip.waveform.len() > 2_048 || clip.waveform.iter().any(|peak| !peak.is_finite()) {
        return Err(CoreError::new("audio clip waveform is invalid"));
    }
    let source_ticks = seconds_to_ticks(clip.source_duration_seconds, bpm);
    if clip.source_offset_ticks.saturating_add(clip.duration_ticks) > source_ticks {
        return Err(CoreError::new(
            "audio clip trim exceeds the source duration",
        ));
    }
    Ok(())
}

fn validate_source_path(path: &str) -> Result<String, CoreError> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(CoreError::new(
            "audio clip source must be a relative project path",
        ));
    }
    Ok(path_string(path))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn seconds_to_ticks(seconds: f64, bpm: f64) -> u32 {
    (seconds * bpm * f64::from(TICKS_PER_QUARTER) / 60.0)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn scale_ticks(ticks: u32, scale: f64) -> Result<u32, CoreError> {
    let scaled = f64::from(ticks) * scale;
    if scaled > f64::from(u32::MAX) {
        return Err(CoreError::new(
            "audio clip range exceeds the supported tick range",
        ));
    }
    Ok(scaled.round() as u32)
}

fn validate_clip_range(start_tick: u32, duration_ticks: u32) -> Result<(), CoreError> {
    if duration_ticks == 0 {
        return Err(CoreError::new(
            "timeline clip duration must be greater than zero",
        ));
    }
    start_tick
        .checked_add(duration_ticks)
        .ok_or_else(|| CoreError::new("timeline clip range exceeds the supported tick range"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{GridDivision, TICKS_PER_QUARTER, Timeline};

    #[test]
    fn creates_and_edits_tracks_and_clips_with_stable_ids() {
        let mut timeline = Timeline::default();
        timeline
            .save_track(Some("track-1"), "Drums", false, false, 0.0, 0.0, true)
            .expect("track should save");
        timeline
            .save_clip(None, "track-1", "Intro", 0, 3_840, 0)
            .expect("clip should save");
        timeline
            .save_clip(Some("clip-1"), "track-1", "Intro", 960, 1_920, 240)
            .expect("clip should update");

        let snapshot = timeline.snapshot();
        assert_eq!(snapshot.tracks[0].id, "track-1");
        assert_eq!(snapshot.tracks[0].clips[0].id, "clip-1");
        assert_eq!(snapshot.tracks[0].clips[0].start_tick, 960);
        assert_eq!(snapshot.tracks[0].clips[0].duration_ticks, 1_920);
        assert_eq!(snapshot.tracks[0].clips[0].source_offset_ticks, 240);

        timeline
            .save_clip(Some("clip-1"), "track-2", "Intro", 960, 1_920, 240)
            .expect("clip should move between tracks");
        let moved = timeline.snapshot();
        assert!(moved.tracks[0].clips.is_empty());
        assert_eq!(moved.tracks[1].clips[0].id, "clip-1");
    }

    #[test]
    fn rejects_invalid_settings_without_mutating_the_timeline() {
        let mut timeline = Timeline::default();

        let error = timeline
            .set_settings(0.0, 4, 4, GridDivision::Eighth, false)
            .expect_err("invalid BPM should be rejected");

        assert_eq!(error.to_string(), "timeline BPM must be between 20 and 400");
        assert!((timeline.snapshot().bpm - 120.0).abs() < f64::EPSILON);
        assert_eq!(timeline.snapshot().grid_division, GridDivision::Quarter);
        assert!(timeline.snapshot().is_snap_enabled);
    }

    #[test]
    fn preserves_audio_clip_time_ranges_when_bpm_changes() {
        let mut timeline = Timeline::default();
        timeline
            .add_audio_clip(
                "track-1",
                "Kick.wav",
                "Kick",
                960,
                7_680,
                48_000,
                2,
                4.0,
                vec![0.5],
            )
            .expect("audio clip should be added");
        timeline
            .save_clip(Some("clip-1"), "track-1", "Kick", 960, 1_920, 960)
            .expect("audio clip should be trimmed");
        timeline
            .save_clip(None, "track-1", "Pattern", 0, 3_840, 0)
            .expect("pattern clip should be added");

        timeline
            .set_settings(60.0, 4, 4, GridDivision::Quarter, true)
            .expect("BPM should update");

        let snapshot = timeline.snapshot();
        let audio = &snapshot.tracks[0].clips[1];
        assert_eq!(audio.start_tick, 960);
        assert_eq!(audio.duration_ticks, 960);
        assert_eq!(audio.source_offset_ticks, 480);
        let pattern = &snapshot.tracks[0].clips[0];
        assert_eq!(pattern.duration_ticks, 3_840);
    }

    #[test]
    fn starts_with_thirty_tracks_and_preserves_monotonic_ids() {
        let mut timeline = Timeline::default();

        assert_eq!(timeline.snapshot().tracks.len(), 30);
        assert_eq!(timeline.snapshot().tracks[0].id, "track-1");
        assert_eq!(timeline.snapshot().tracks[29].id, "track-30");

        timeline
            .save_track(None, "Track 31", false, false, 0.0, 0.0, true)
            .expect("track should save");
        assert_eq!(timeline.snapshot().tracks[30].id, "track-31");
    }

    #[test]
    fn positions_new_tracks_by_their_monotonic_id() {
        let mut timeline = Timeline::default();
        timeline
            .delete_track("track-1")
            .expect("existing track should delete");
        timeline
            .save_track(None, "Track 31", false, false, 0.0, 0.0, true)
            .expect("track should save");

        let track = timeline
            .snapshot()
            .tracks
            .into_iter()
            .find(|track| track.id == "track-31")
            .expect("new track should exist");
        assert!(track.node_x.abs() < f64::EPSILON);
        assert!((track.node_y - 1_080.0).abs() < f64::EPSILON);
    }

    #[test]
    fn prevents_deleting_the_final_track() {
        let mut timeline = Timeline::default();
        while timeline.snapshot().tracks.len() > 1 {
            let id = timeline.snapshot().tracks[0].id.clone();
            timeline.delete_track(&id).expect("track should delete");
        }

        let final_id = timeline.snapshot().tracks[0].id.clone();
        let error = timeline
            .delete_track(&final_id)
            .expect_err("final track should remain");

        assert_eq!(
            error.to_string(),
            "a project must contain at least one timeline track"
        );
        assert_eq!(timeline.snapshot().tracks.len(), 1);
    }

    #[test]
    fn exposes_exact_tick_sizes_for_straight_and_triplet_grids() {
        assert_eq!(GridDivision::Quarter.ticks(), TICKS_PER_QUARTER);
        assert_eq!(GridDivision::Eighth.ticks(), 480);
        assert_eq!(GridDivision::EighthTriplet.ticks(), 320);
        assert_eq!(GridDivision::SixteenthTriplet.ticks(), 160);
    }
}
