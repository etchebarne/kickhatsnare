use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::mix_graph::{
    LegacyTrackMix, MixGraph, MixGraphSnapshot, default_master_x, default_master_y,
    default_track_position_for,
};
use crate::CoreError;

pub const TICKS_PER_QUARTER: u32 = 960;

const DEFAULT_BPM: f64 = 120.0;
const DEFAULT_TRACK_COUNT: u64 = 10;
const DEFAULT_TIME_SIGNATURE_NUMERATOR: u8 = 4;
const DEFAULT_TIME_SIGNATURE_DENOMINATOR: u8 = 4;

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipStretchMode {
    #[default]
    Resample,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipResizeMode {
    Trim,
    Stretch,
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
    pub mix_graph: MixGraphSnapshot,
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
    pub clips: Vec<TimelineClipSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClipSnapshot {
    pub id: String,
    pub name: String,
    pub start_tick: u32,
    pub duration_ticks: u32,
    pub source_offset_ticks: u32,
    pub source_duration_ticks: u32,
    pub source_path: Option<String>,
    pub source_sample_rate: u32,
    pub source_channels: u16,
    pub source_duration_seconds: f64,
    pub waveform: Vec<f32>,
    pub stretch_mode: ClipStretchMode,
    pub gain_db: f64,
    pub pan: f64,
    pub pitch_semitones: f64,
    pub tempo_percent: f64,
    pub is_unique: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
    #[serde(default, rename = "masterNodeX", skip_serializing)]
    legacy_master_node_x: Option<f64>,
    #[serde(default, rename = "masterNodeY", skip_serializing)]
    legacy_master_node_y: Option<f64>,
    #[serde(default)]
    mix_graph: MixGraph,
    tracks: Vec<TimelineTrack>,
    next_track_id: u64,
    next_clip_id: u64,
    #[serde(default = "default_next_clip_settings_id")]
    next_clip_settings_id: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
    #[serde(default, rename = "isConnected", skip_serializing)]
    legacy_is_connected: Option<bool>,
    #[serde(default, rename = "nodeX", skip_serializing)]
    legacy_node_x: Option<f64>,
    #[serde(default, rename = "nodeY", skip_serializing)]
    legacy_node_y: Option<f64>,
    clips: Vec<TimelineClip>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TimelineClip {
    id: String,
    name: String,
    start_tick: u32,
    duration_ticks: u32,
    source_offset_ticks: u32,
    #[serde(default)]
    source_duration_ticks: u32,
    #[serde(default)]
    source_path: Option<String>,
    #[serde(default)]
    source_sample_rate: u32,
    #[serde(default)]
    source_channels: u16,
    #[serde(default)]
    source_duration_seconds: f64,
    #[serde(default)]
    waveform: Arc<Vec<f32>>,
    #[serde(default)]
    settings_id: String,
    #[serde(default)]
    settings: TimelineClipSettings,
    #[serde(default)]
    is_unique: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TimelineClipSettings {
    stretch_mode: ClipStretchMode,
    gain_db: f64,
    pan: f64,
    pitch_semitones: f64,
    #[serde(default = "default_tempo_percent")]
    tempo_percent: f64,
}

impl Default for TimelineClipSettings {
    fn default() -> Self {
        Self {
            stretch_mode: ClipStretchMode::default(),
            gain_db: 0.0,
            pan: 0.0,
            pitch_semitones: 0.0,
            tempo_percent: default_tempo_percent(),
        }
    }
}

impl Default for Timeline {
    fn default() -> Self {
        let tracks = (1..=DEFAULT_TRACK_COUNT)
            .map(default_track)
            .collect::<Vec<_>>();
        let mix_graph = MixGraph::new(tracks.iter().map(|track| track.id.clone()));
        Self {
            bpm: DEFAULT_BPM,
            time_signature_numerator: DEFAULT_TIME_SIGNATURE_NUMERATOR,
            time_signature_denominator: DEFAULT_TIME_SIGNATURE_DENOMINATOR,
            grid_division: GridDivision::Quarter,
            is_snap_enabled: true,
            master_gain_db: 0.0,
            is_master_muted: false,
            legacy_master_node_x: None,
            legacy_master_node_y: None,
            mix_graph,
            tracks,
            next_track_id: DEFAULT_TRACK_COUNT + 1,
            next_clip_id: 1,
            next_clip_settings_id: 1,
        }
    }
}

impl Timeline {
    pub(super) fn bpm(&self) -> f64 {
        self.bpm
    }

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
            mix_graph: self.mix_graph.snapshot(),
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
                    clips: track
                        .clips
                        .iter()
                        .map(|clip| TimelineClipSnapshot {
                            id: clip.id.clone(),
                            name: clip.name.clone(),
                            start_tick: clip.start_tick,
                            duration_ticks: clip.duration_ticks,
                            source_offset_ticks: clip.source_offset_ticks,
                            source_duration_ticks: clip.source_duration_ticks,
                            source_path: clip.source_path.clone(),
                            source_sample_rate: clip.source_sample_rate,
                            source_channels: clip.source_channels,
                            source_duration_seconds: clip.source_duration_seconds,
                            waveform: clip.waveform.as_ref().clone(),
                            stretch_mode: clip.settings.stretch_mode,
                            gain_db: clip.settings.gain_db,
                            pan: clip.settings.pan,
                            pitch_semitones: clip.settings.pitch_semitones,
                            tempo_percent: clip.settings.tempo_percent,
                            is_unique: clip.is_unique,
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
        let mut track_ids = HashSet::new();
        let mut clip_ids = HashSet::new();
        let mut clip_settings = HashMap::new();
        for track in &self.tracks {
            validate_name(&track.name, "track")?;
            validate_gain(track.gain_db, "track")?;
            validate_pan(track.pan)?;
            if track.id.is_empty() || !track_ids.insert(track.id.as_str()) {
                return Err(CoreError::new(
                    "project contains an invalid or duplicate track ID",
                ));
            }
            for clip in &track.clips {
                validate_name(&clip.name, "clip")?;
                validate_clip_range(clip.start_tick, clip.duration_ticks)?;
                validate_source(clip, self.bpm)?;
                validate_clip_settings(&clip.settings)?;
                if clip.settings_id.is_empty() {
                    return Err(CoreError::new(
                        "project contains an invalid clip settings ID",
                    ));
                }
                if let Some((settings, is_unique, source_path)) =
                    clip_settings.get(clip.settings_id.as_str())
                    && (*settings != &clip.settings
                        || *is_unique != clip.is_unique
                        || *source_path != clip.source_path.as_deref())
                {
                    return Err(CoreError::new(
                        "project contains inconsistent shared clip settings",
                    ));
                }
                clip_settings.insert(
                    clip.settings_id.as_str(),
                    (&clip.settings, clip.is_unique, clip.source_path.as_deref()),
                );
                if clip.id.is_empty() || !clip_ids.insert(clip.id.as_str()) {
                    return Err(CoreError::new(
                        "project contains an invalid or duplicate clip ID",
                    ));
                }
            }
        }
        self.mix_graph.validate(&track_ids)?;
        Ok(())
    }

    pub(super) fn ensure_minimum_track(&mut self) -> Result<(), CoreError> {
        if self.tracks.is_empty() {
            let id = self.next_track_id()?;
            self.mix_graph.add_track(&id)?;
            self.tracks.push(TimelineTrack {
                id: id.clone(),
                name: "Track 1".to_owned(),
                is_muted: false,
                is_soloed: false,
                gain_db: 0.0,
                pan: 0.0,
                legacy_is_connected: None,
                legacy_node_x: None,
                legacy_node_y: None,
                clips: Vec::new(),
            });
        }
        Ok(())
    }

    pub(super) fn migrate_from(&mut self, format_version: u32) {
        if format_version < 4 {
            let tracks = self
                .tracks
                .iter()
                .enumerate()
                .map(|(index, track)| {
                    let (default_x, default_y) = default_track_position_for(index as u64 + 1);
                    LegacyTrackMix {
                        id: track.id.clone(),
                        is_connected: track.legacy_is_connected.unwrap_or(true),
                        x: track.legacy_node_x.unwrap_or(default_x),
                        y: track.legacy_node_y.unwrap_or(default_y),
                    }
                })
                .collect::<Vec<_>>();
            self.mix_graph = MixGraph::from_legacy(
                tracks,
                self.legacy_master_node_x.unwrap_or_else(default_master_x),
                self.legacy_master_node_y.unwrap_or_else(default_master_y),
            );
            self.legacy_master_node_x = None;
            self.legacy_master_node_y = None;
            for track in &mut self.tracks {
                track.legacy_is_connected = None;
                track.legacy_node_x = None;
                track.legacy_node_y = None;
            }
        }
        if format_version < 5 {
            let mut source_settings = HashMap::new();
            let mut next_settings_id = 1_u64;
            for clip in self.tracks.iter_mut().flat_map(|track| &mut track.clips) {
                clip.source_duration_ticks = clip.duration_ticks;
                let key = clip
                    .source_path
                    .clone()
                    .unwrap_or_else(|| format!("region:{}", clip.id));
                clip.settings_id
                    .clone_from(source_settings.entry(key).or_insert_with(|| {
                        let id = format!("clip-settings-{next_settings_id}");
                        next_settings_id = next_settings_id.saturating_add(1);
                        id
                    }));
                clip.settings = TimelineClipSettings::default();
                clip.is_unique = clip.source_path.is_none();
            }
            self.next_clip_settings_id = next_settings_id;
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
                clip.source_duration_ticks = scale_ticks(clip.source_duration_ticks, scale)?.max(1);

                let source_ticks = seconds_to_ticks(clip.source_duration_seconds, bpm).max(1);
                if clip.source_offset_ticks >= source_ticks {
                    return Err(CoreError::new(
                        "audio clip trim exceeds the source duration",
                    ));
                }
                clip.source_duration_ticks = clip
                    .source_duration_ticks
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

    pub(super) fn save_track(
        &mut self,
        id: Option<&str>,
        name: &str,
        is_muted: bool,
        is_soloed: bool,
        gain_db: f64,
        pan: f64,
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
        } else {
            let id = self.next_track_id()?;
            self.mix_graph.add_track(&id)?;
            self.tracks.push(TimelineTrack {
                id: id.clone(),
                name,
                is_muted,
                is_soloed,
                gain_db,
                pan,
                legacy_is_connected: None,
                legacy_node_x: None,
                legacy_node_y: None,
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
        self.mix_graph.remove_track(id);
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
        source_duration_ticks: u32,
        resize_mode: ClipResizeMode,
    ) -> Result<(), CoreError> {
        let name = validate_name(name, "clip")?;
        validate_clip_range(start_tick, duration_ticks)?;
        if source_duration_ticks == 0 {
            return Err(CoreError::new(
                "timeline clip source duration must be greater than zero",
            ));
        }
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
            let is_audio = existing.source_path.is_some();
            let settings_id = existing.settings_id.clone();
            let duration_changed = existing.duration_ticks != duration_ticks
                || existing.source_duration_ticks != source_duration_ticks;
            if is_audio {
                let source_ticks = seconds_to_ticks(existing.source_duration_seconds, self.bpm);
                if source_offset_ticks.saturating_add(source_duration_ticks) > source_ticks {
                    return Err(CoreError::new(
                        "audio clip trim exceeds the source duration",
                    ));
                }
            }
            if is_audio && duration_changed && resize_mode == ClipResizeMode::Stretch {
                self.propagate_stretch_tempo(
                    id,
                    &settings_id,
                    source_duration_ticks,
                    duration_ticks,
                )?;
            }
            let mut clip = self.tracks[source_track_index].clips.remove(clip_index);
            clip.name = name;
            clip.start_tick = start_tick;
            clip.duration_ticks = duration_ticks;
            clip.source_offset_ticks = source_offset_ticks;
            clip.source_duration_ticks = if clip.source_path.is_some() {
                source_duration_ticks
            } else {
                duration_ticks
            };
            self.tracks[target_track_index].clips.push(clip);
        } else {
            let id = self.next_clip_id()?;
            let settings_id = self.next_clip_settings_id()?;
            self.tracks[target_track_index].clips.push(TimelineClip {
                id,
                name,
                start_tick,
                duration_ticks,
                source_offset_ticks,
                source_duration_ticks: duration_ticks,
                source_path: None,
                source_sample_rate: 0,
                source_channels: 0,
                source_duration_seconds: 0.0,
                waveform: Arc::new(Vec::new()),
                settings_id,
                settings: TimelineClipSettings::default(),
                is_unique: true,
            });
        }
        self.tracks[target_track_index]
            .clips
            .sort_by_key(|clip| (clip.start_tick, clip.id.clone()));
        Ok(())
    }

    pub(super) fn split_clip(&mut self, id: &str, split_tick: u32) -> Result<(), CoreError> {
        let (track_index, clip_index) = self
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
        let clip = &self.tracks[track_index].clips[clip_index];
        let end_tick = clip
            .start_tick
            .checked_add(clip.duration_ticks)
            .ok_or_else(|| {
                CoreError::new("timeline clip range exceeds the supported tick range")
            })?;
        if split_tick <= clip.start_tick || split_tick >= end_tick {
            return Err(CoreError::new("split point must be inside timeline clip"));
        }
        if clip.source_path.is_some() && clip.source_duration_ticks < 2 {
            return Err(CoreError::new(
                "audio clip source window is too short to split",
            ));
        }
        let left_duration = split_tick - clip.start_tick;
        let right_duration = end_tick - split_tick;
        let (left_source_duration, consumed_source_duration, right_source_duration) =
            split_source_duration(
                clip.source_duration_ticks,
                left_duration,
                clip.duration_ticks,
            );
        let right_source_offset = clip
            .source_offset_ticks
            .checked_add(consumed_source_duration)
            .ok_or_else(|| {
                CoreError::new("timeline clip source range exceeds the supported tick range")
            })?;
        let right_id = self.next_clip_id()?;
        let right = {
            let left = &mut self.tracks[track_index].clips[clip_index];
            let mut right = left.clone();
            left.duration_ticks = left_duration;
            left.source_duration_ticks = left_source_duration;
            right.id = right_id;
            right.start_tick = split_tick;
            right.duration_ticks = right_duration;
            right.source_offset_ticks = right_source_offset;
            right.source_duration_ticks = right_source_duration;
            right
        };
        self.tracks[track_index].clips.push(right);
        self.tracks[track_index]
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
        let shared_settings = self
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .find(|clip| {
                !clip.is_unique && clip.source_path.as_deref() == Some(source_path.as_str())
            })
            .map(|clip| (clip.settings_id.clone(), clip.settings.clone()));
        let (settings_id, settings) = if let Some(shared) = shared_settings {
            shared
        } else {
            (
                self.next_clip_settings_id()?,
                TimelineClipSettings::default(),
            )
        };
        let clip_duration_ticks =
            tempo_duration_ticks(duration_ticks, settings.tempo_percent, start_tick)?;
        self.tracks[target_track_index].clips.push(TimelineClip {
            id,
            name,
            start_tick,
            duration_ticks: clip_duration_ticks,
            source_offset_ticks: 0,
            source_duration_ticks: duration_ticks,
            source_path: Some(source_path),
            source_sample_rate: sample_rate,
            source_channels: channels,
            source_duration_seconds: duration_seconds,
            waveform: Arc::new(waveform),
            settings_id,
            settings,
            is_unique: false,
        });
        self.tracks[target_track_index]
            .clips
            .sort_by_key(|clip| (clip.start_tick, clip.id.clone()));
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn set_clip_properties(
        &mut self,
        id: &str,
        stretch_mode: ClipStretchMode,
        gain_db: f64,
        pan: f64,
        pitch_semitones: f64,
        tempo_percent: Option<f64>,
        make_unique: bool,
    ) -> Result<(), CoreError> {
        if tempo_percent.is_some_and(|tempo| !tempo.is_finite() || !(25.0..=400.0).contains(&tempo))
        {
            return Err(CoreError::new("clip tempo must be between 25% and 400%"));
        }

        let mut updated = self.clone();
        let (track_index, clip_index) = updated
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
        if updated.tracks[track_index].clips[clip_index]
            .source_path
            .is_none()
        {
            return Err(CoreError::new("timeline clip does not contain audio"));
        }
        let settings = TimelineClipSettings {
            stretch_mode,
            gain_db,
            pan,
            pitch_semitones,
            tempo_percent: tempo_percent.unwrap_or(
                updated.tracks[track_index].clips[clip_index]
                    .settings
                    .tempo_percent,
            ),
        };
        validate_clip_settings(&settings)?;
        if make_unique && !updated.tracks[track_index].clips[clip_index].is_unique {
            let settings_id = updated.next_clip_settings_id()?;
            let clip = &mut updated.tracks[track_index].clips[clip_index];
            clip.settings_id = settings_id;
            clip.is_unique = true;
        }
        let settings_id = updated.tracks[track_index].clips[clip_index]
            .settings_id
            .clone();
        for clip in updated
            .tracks
            .iter_mut()
            .flat_map(|track| &mut track.clips)
            .filter(|clip| clip.settings_id == settings_id)
        {
            clip.settings = settings.clone();
            if let Some(tempo_percent) = tempo_percent {
                clip.duration_ticks = tempo_duration_ticks(
                    clip.source_duration_ticks,
                    tempo_percent,
                    clip.start_tick,
                )?;
            }
        }
        updated.validate()?;
        *self = updated;
        Ok(())
    }

    fn propagate_stretch_tempo(
        &mut self,
        edited_clip_id: &str,
        settings_id: &str,
        source_duration_ticks: u32,
        duration_ticks: u32,
    ) -> Result<(), CoreError> {
        let tempo_percent = f64::from(source_duration_ticks) * 100.0 / f64::from(duration_ticks);
        if !(25.0..=400.0).contains(&tempo_percent) {
            return Err(CoreError::new("clip tempo must be between 25% and 400%"));
        }
        let resized = self
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .filter(|clip| clip.settings_id == settings_id && clip.id != edited_clip_id)
            .map(|clip| {
                tempo_duration_ticks(clip.source_duration_ticks, tempo_percent, clip.start_tick)
                    .map(|duration| (clip.id.clone(), duration))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for clip in self.tracks.iter_mut().flat_map(|track| &mut track.clips) {
            if clip.settings_id == settings_id {
                clip.settings.tempo_percent = tempo_percent;
            }
            if let Some((_, duration)) = resized.iter().find(|(clip_id, _)| clip_id == &clip.id) {
                clip.duration_ticks = *duration;
            }
        }
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
        self.mix_graph.set_node_position(node_id, x, y)
    }

    pub(super) fn connect_mix_ports(
        &mut self,
        source_node_id: &str,
        source_port_id: &str,
        target_node_id: &str,
        target_port_id: &str,
    ) -> Result<(), CoreError> {
        self.mix_graph.connect(
            source_node_id,
            source_port_id,
            target_node_id,
            target_port_id,
        )
    }

    pub(super) fn disconnect_mix_ports(
        &mut self,
        source_node_id: &str,
        source_port_id: &str,
        target_node_id: &str,
        target_port_id: &str,
    ) -> Result<(), CoreError> {
        self.mix_graph.disconnect(
            source_node_id,
            source_port_id,
            target_node_id,
            target_port_id,
        )
    }

    pub(super) fn track_routes_to_master(&self, track_id: &str) -> bool {
        self.mix_graph.track_routes_to_master(track_id)
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

    fn next_clip_settings_id(&mut self) -> Result<String, CoreError> {
        loop {
            let id = format!("clip-settings-{}", self.next_clip_settings_id);
            self.next_clip_settings_id = self
                .next_clip_settings_id
                .checked_add(1)
                .ok_or_else(|| CoreError::new("timeline clip settings ID space is exhausted"))?;
            if self
                .tracks
                .iter()
                .flat_map(|track| &track.clips)
                .all(|clip| clip.settings_id != id)
            {
                return Ok(id);
            }
        }
    }
}

fn default_track(number: u64) -> TimelineTrack {
    TimelineTrack {
        id: format!("track-{number}"),
        name: format!("Track {number}"),
        is_muted: false,
        is_soloed: false,
        gain_db: 0.0,
        pan: 0.0,
        legacy_is_connected: None,
        legacy_node_x: None,
        legacy_node_y: None,
        clips: Vec::new(),
    }
}

const fn default_next_clip_settings_id() -> u64 {
    1
}

const fn default_tempo_percent() -> f64 {
    100.0
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
        return Err(CoreError::new("pan must be between -1 and 1"));
    }
    Ok(())
}

fn validate_clip_settings(settings: &TimelineClipSettings) -> Result<(), CoreError> {
    validate_gain(settings.gain_db, "clip")?;
    validate_pan(settings.pan)?;
    if !settings.pitch_semitones.is_finite() || !(-24.0..=24.0).contains(&settings.pitch_semitones)
    {
        return Err(CoreError::new(
            "clip pitch must be between -24 and 24 semitones",
        ));
    }
    if !settings.tempo_percent.is_finite() || !(25.0..=400.0).contains(&settings.tempo_percent) {
        return Err(CoreError::new("clip tempo must be between 25% and 400%"));
    }
    Ok(())
}

fn validate_source(clip: &TimelineClip, bpm: f64) -> Result<(), CoreError> {
    if clip.source_duration_ticks == 0 {
        return Err(CoreError::new(
            "timeline clip source duration must be greater than zero",
        ));
    }
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
    if clip
        .source_offset_ticks
        .saturating_add(clip.source_duration_ticks)
        > source_ticks
    {
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

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn tempo_duration_ticks(
    source_duration_ticks: u32,
    tempo_percent: f64,
    start_tick: u32,
) -> Result<u32, CoreError> {
    let duration = (f64::from(source_duration_ticks) * 100.0 / tempo_percent)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32;
    validate_clip_range(start_tick, duration)?;
    Ok(duration)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn split_source_duration(
    source_duration_ticks: u32,
    left_duration_ticks: u32,
    duration_ticks: u32,
) -> (u32, u32, u32) {
    debug_assert!(source_duration_ticks > 1);
    let consumed = (f64::from(source_duration_ticks) * f64::from(left_duration_ticks)
        / f64::from(duration_ticks))
    .round()
    .clamp(1.0, f64::from(source_duration_ticks - 1)) as u32;
    (consumed, consumed, source_duration_ticks - consumed)
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
    use std::sync::Arc;

    use super::{
        ClipResizeMode, ClipStretchMode, GridDivision, TICKS_PER_QUARTER, Timeline,
        TimelineClipSettings,
    };

    #[test]
    fn creates_and_edits_tracks_and_clips_with_stable_ids() {
        let mut timeline = Timeline::default();
        timeline
            .save_track(Some("track-1"), "Drums", false, false, 0.0, 0.0)
            .expect("track should save");
        timeline
            .save_clip(
                None,
                "track-1",
                "Intro",
                0,
                3_840,
                0,
                3_840,
                ClipResizeMode::Trim,
            )
            .expect("clip should save");
        timeline
            .save_clip(
                Some("clip-1"),
                "track-1",
                "Intro",
                960,
                1_920,
                240,
                1_920,
                ClipResizeMode::Trim,
            )
            .expect("clip should update");

        let snapshot = timeline.snapshot();
        assert_eq!(snapshot.tracks[0].id, "track-1");
        assert_eq!(snapshot.tracks[0].clips[0].id, "clip-1");
        assert_eq!(snapshot.tracks[0].clips[0].start_tick, 960);
        assert_eq!(snapshot.tracks[0].clips[0].duration_ticks, 1_920);
        assert_eq!(snapshot.tracks[0].clips[0].source_offset_ticks, 240);

        timeline
            .save_clip(
                Some("clip-1"),
                "track-2",
                "Intro",
                960,
                1_920,
                240,
                1_920,
                ClipResizeMode::Trim,
            )
            .expect("clip should move between tracks");
        let moved = timeline.snapshot();
        assert!(moved.tracks[0].clips.is_empty());
        assert_eq!(moved.tracks[1].clips[0].id, "clip-1");
    }

    #[test]
    fn splits_audio_clips_without_copying_source_data() {
        let mut timeline = Timeline::default();
        timeline
            .add_audio_clip(
                "track-1",
                "Kick.wav",
                "Kick",
                960,
                3_840,
                48_000,
                2,
                2.0,
                vec![0.25, 0.5],
            )
            .expect("audio clip should be added");

        timeline
            .split_clip("clip-1", 2_880)
            .expect("audio clip should split");

        let clips = &timeline.tracks[0].clips;
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].id, "clip-1");
        assert_eq!(clips[0].start_tick, 960);
        assert_eq!(clips[0].duration_ticks, 1_920);
        assert_eq!(clips[0].source_offset_ticks, 0);
        assert_eq!(clips[0].source_duration_ticks, 1_920);
        assert_eq!(clips[1].id, "clip-2");
        assert_eq!(clips[1].name, "Kick");
        assert_eq!(clips[1].start_tick, 2_880);
        assert_eq!(clips[1].duration_ticks, 1_920);
        assert_eq!(clips[1].source_offset_ticks, 1_920);
        assert_eq!(clips[1].source_duration_ticks, 1_920);
        assert_eq!(clips[1].source_path.as_deref(), Some("Kick.wav"));
        assert!(Arc::ptr_eq(&clips[0].waveform, &clips[1].waveform));
    }

    #[test]
    fn shares_source_settings_until_a_clip_is_made_unique() {
        let mut timeline = Timeline::default();
        for start_tick in [0, 3_840] {
            timeline
                .add_audio_clip(
                    "track-1",
                    "Loop.wav",
                    "Loop",
                    start_tick,
                    3_840,
                    48_000,
                    2,
                    2.0,
                    vec![0.5],
                )
                .expect("audio clip should be added");
        }

        timeline
            .set_clip_properties(
                "clip-1",
                ClipStretchMode::Stretch,
                -3.0,
                0.25,
                5.0,
                Some(200.0),
                false,
            )
            .expect("shared properties should update");
        let shared = timeline.snapshot();
        let clips = &shared.tracks[0].clips;
        assert_eq!(clips[0].stretch_mode, ClipStretchMode::Stretch);
        assert_eq!(clips[1].stretch_mode, ClipStretchMode::Stretch);
        assert_eq!(clips[0].duration_ticks, 1_920);
        assert_eq!(clips[1].duration_ticks, 1_920);
        assert!((clips[0].tempo_percent - 200.0).abs() < f64::EPSILON);

        timeline
            .add_audio_clip(
                "track-1",
                "Loop.wav",
                "Loop",
                7_680,
                3_840,
                48_000,
                2,
                2.0,
                vec![0.5],
            )
            .expect("shared source should inherit settings");
        assert_eq!(timeline.snapshot().tracks[0].clips[2].duration_ticks, 1_920);

        timeline
            .save_clip(
                Some("clip-1"),
                "track-1",
                "Loop",
                0,
                3_840,
                0,
                3_840,
                ClipResizeMode::Stretch,
            )
            .expect("stretch resize should update shared tempo");
        let resized = timeline.snapshot();
        assert!(resized.tracks[0].clips.iter().all(|clip| {
            clip.duration_ticks == 3_840 && (clip.tempo_percent - 100.0).abs() < f64::EPSILON
        }));

        timeline
            .set_clip_properties(
                "clip-2",
                ClipStretchMode::Resample,
                0.0,
                0.0,
                0.0,
                None,
                true,
            )
            .expect("clip should become unique");
        let unique = timeline.snapshot();
        let clips = &unique.tracks[0].clips;
        assert_eq!(clips[0].stretch_mode, ClipStretchMode::Stretch);
        assert_eq!(clips[1].stretch_mode, ClipStretchMode::Resample);
        assert!(!clips[0].is_unique);
        assert!(clips[1].is_unique);
    }

    #[test]
    fn migrates_version_four_audio_clips_to_shared_neutral_settings() {
        let mut timeline: Timeline = serde_json::from_value(serde_json::json!({
            "bpm": 120.0,
            "timeSignatureNumerator": 4,
            "timeSignatureDenominator": 4,
            "gridDivision": "quarter",
            "isSnapEnabled": true,
            "tracks": [{
                "id": "track-1",
                "name": "Track 1",
                "isMuted": false,
                "isSoloed": false,
                "clips": [
                    {
                        "id": "clip-1",
                        "name": "Loop",
                        "startTick": 0,
                        "durationTicks": 3840,
                        "sourceOffsetTicks": 0,
                        "sourcePath": "Loop.wav",
                        "sourceSampleRate": 48000,
                        "sourceChannels": 2,
                        "sourceDurationSeconds": 2.0,
                        "waveform": [0.5]
                    },
                    {
                        "id": "clip-2",
                        "name": "Loop",
                        "startTick": 3840,
                        "durationTicks": 3840,
                        "sourceOffsetTicks": 0,
                        "sourcePath": "Loop.wav",
                        "sourceSampleRate": 48000,
                        "sourceChannels": 2,
                        "sourceDurationSeconds": 2.0,
                        "waveform": [0.5]
                    }
                ]
            }],
            "nextTrackId": 2,
            "nextClipId": 3
        }))
        .expect("version four timeline should deserialize");

        timeline.migrate_from(4);

        let clips = &timeline.tracks[0].clips;
        assert_eq!(clips[0].source_duration_ticks, 3_840);
        assert_eq!(clips[0].settings_id, clips[1].settings_id);
        assert_eq!(clips[0].settings, TimelineClipSettings::default());
        assert!(!clips[0].is_unique);
    }

    #[test]
    fn splits_stretched_clips_at_the_matching_source_position() {
        let mut timeline = Timeline::default();
        timeline
            .add_audio_clip(
                "track-1",
                "Loop.wav",
                "Loop",
                0,
                3_840,
                48_000,
                2,
                2.0,
                vec![0.5],
            )
            .expect("audio clip should be added");
        timeline
            .save_clip(
                Some("clip-1"),
                "track-1",
                "Loop",
                0,
                1_920,
                0,
                3_840,
                ClipResizeMode::Stretch,
            )
            .expect("clip should stretch");

        timeline
            .split_clip("clip-1", 960)
            .expect("stretched clip should split");

        let snapshot = timeline.snapshot();
        let clips = &snapshot.tracks[0].clips;
        assert_eq!(clips[0].duration_ticks, 960);
        assert_eq!(clips[0].source_duration_ticks, 1_920);
        assert_eq!(clips[1].source_offset_ticks, 1_920);
        assert_eq!(clips[1].source_duration_ticks, 1_920);
    }

    #[test]
    fn rejects_split_points_at_clip_boundaries_without_mutating() {
        let mut timeline = Timeline::default();
        timeline
            .save_clip(
                None,
                "track-1",
                "Region",
                960,
                1_920,
                0,
                1_920,
                ClipResizeMode::Trim,
            )
            .expect("clip should save");
        let before = timeline.clone();

        let start_error = timeline
            .split_clip("clip-1", 960)
            .expect_err("clip start should not split");
        let end_error = timeline
            .split_clip("clip-1", 2_880)
            .expect_err("clip end should not split");

        assert_eq!(
            start_error.to_string(),
            "split point must be inside timeline clip"
        );
        assert_eq!(
            end_error.to_string(),
            "split point must be inside timeline clip"
        );
        assert_eq!(timeline, before);
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
            .save_clip(
                Some("clip-1"),
                "track-1",
                "Kick",
                960,
                1_920,
                960,
                1_920,
                ClipResizeMode::Trim,
            )
            .expect("audio clip should be trimmed");
        timeline
            .save_clip(
                None,
                "track-1",
                "Pattern",
                0,
                3_840,
                0,
                3_840,
                ClipResizeMode::Trim,
            )
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
    fn starts_with_ten_tracks_and_preserves_monotonic_ids() {
        let mut timeline = Timeline::default();

        assert_eq!(timeline.snapshot().tracks.len(), 10);
        assert_eq!(timeline.snapshot().tracks[0].id, "track-1");
        assert_eq!(timeline.snapshot().tracks[9].id, "track-10");

        timeline
            .save_track(None, "Track 11", false, false, 0.0, 0.0)
            .expect("track should save");
        assert_eq!(timeline.snapshot().tracks[10].id, "track-11");
    }

    #[test]
    fn positions_new_tracks_by_their_monotonic_id() {
        let mut timeline = Timeline::default();
        timeline
            .delete_track("track-1")
            .expect("existing track should delete");
        timeline
            .save_track(None, "Track 11", false, false, 0.0, 0.0)
            .expect("track should save");

        let node = timeline
            .snapshot()
            .mix_graph
            .nodes
            .into_iter()
            .find(|node| node.id == "track-11")
            .expect("new track node should exist");
        assert!(node.x.abs() < f64::EPSILON);
        assert!((node.y - 880.0).abs() < f64::EPSILON);
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
