use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::CoreError;

pub const TICKS_PER_QUARTER: u32 = 960;

const DEFAULT_BPM: f64 = 120.0;
const DEFAULT_TRACK_COUNT: u64 = 30;
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
    pub tracks: Vec<TimelineTrackSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineTrackSnapshot {
    pub id: String,
    pub name: String,
    pub is_muted: bool,
    pub is_soloed: bool,
    pub clips: Vec<TimelineClipSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineClipSnapshot {
    pub id: String,
    pub name: String,
    pub start_tick: u32,
    pub duration_ticks: u32,
    pub source_offset_ticks: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Timeline {
    bpm: f64,
    time_signature_numerator: u8,
    time_signature_denominator: u8,
    grid_division: GridDivision,
    is_snap_enabled: bool,
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
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            bpm: DEFAULT_BPM,
            time_signature_numerator: DEFAULT_TIME_SIGNATURE_NUMERATOR,
            time_signature_denominator: DEFAULT_TIME_SIGNATURE_DENOMINATOR,
            grid_division: GridDivision::Quarter,
            is_snap_enabled: true,
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
            tracks: self
                .tracks
                .iter()
                .map(|track| TimelineTrackSnapshot {
                    id: track.id.clone(),
                    name: track.name.clone(),
                    is_muted: track.is_muted,
                    is_soloed: track.is_soloed,
                    clips: track
                        .clips
                        .iter()
                        .map(|clip| TimelineClipSnapshot {
                            id: clip.id.clone(),
                            name: clip.name.clone(),
                            start_tick: clip.start_tick,
                            duration_ticks: clip.duration_ticks,
                            source_offset_ticks: clip.source_offset_ticks,
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
        let mut track_ids = HashSet::new();
        let mut clip_ids = HashSet::new();
        for track in &self.tracks {
            validate_name(&track.name, "track")?;
            if track.id.is_empty() || !track_ids.insert(track.id.as_str()) {
                return Err(CoreError::new(
                    "project contains an invalid or duplicate track ID",
                ));
            }
            for clip in &track.clips {
                validate_name(&clip.name, "clip")?;
                validate_clip_range(clip.start_tick, clip.duration_ticks)?;
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
                clips: Vec::new(),
            });
        }
        Ok(())
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
        self.bpm = bpm;
        self.time_signature_numerator = time_signature_numerator;
        self.time_signature_denominator = time_signature_denominator;
        self.grid_division = grid_division;
        self.is_snap_enabled = is_snap_enabled;
        Ok(())
    }

    pub(super) fn save_track(
        &mut self,
        id: Option<&str>,
        name: &str,
        is_muted: bool,
        is_soloed: bool,
    ) -> Result<(), CoreError> {
        let name = validate_name(name, "track")?;
        if let Some(id) = id {
            let track = self
                .tracks
                .iter_mut()
                .find(|track| track.id == id)
                .ok_or_else(|| CoreError::new("timeline track does not exist"))?;
            track.name = name;
            track.is_muted = is_muted;
            track.is_soloed = is_soloed;
        } else {
            let id = self.next_track_id()?;
            self.tracks.push(TimelineTrack {
                id,
                name,
                is_muted,
                is_soloed,
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
    TimelineTrack {
        id: format!("track-{number}"),
        name: format!("Track {number}"),
        is_muted: false,
        is_soloed: false,
        clips: Vec::new(),
    }
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
            .save_track(Some("track-1"), "Drums", false, false)
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
    fn starts_with_thirty_tracks_and_preserves_monotonic_ids() {
        let mut timeline = Timeline::default();

        assert_eq!(timeline.snapshot().tracks.len(), 30);
        assert_eq!(timeline.snapshot().tracks[0].id, "track-1");
        assert_eq!(timeline.snapshot().tracks[29].id, "track-30");

        timeline
            .save_track(None, "Track 31", false, false)
            .expect("track should save");
        assert_eq!(timeline.snapshot().tracks[30].id, "track-31");
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
