mod create_directory;
mod delete_entry;
mod delete_timeline_clip;
mod delete_timeline_track;
mod get;
mod import_audio;
mod move_entry;
mod new;
mod open;
mod save;
mod save_as;
mod save_timeline_clip;
mod save_timeline_track;
mod set_timeline_settings;

pub use create_directory::{CreateWorkspaceDirectory, CreateWorkspaceDirectoryParams};
pub use delete_entry::{DeleteWorkspaceEntry, DeleteWorkspaceEntryParams};
pub use delete_timeline_clip::{DeleteTimelineClip, DeleteTimelineClipParams};
pub use delete_timeline_track::{DeleteTimelineTrack, DeleteTimelineTrackParams};
pub use get::{GetWorkspace, GetWorkspaceParams};
pub use import_audio::{ImportWorkspaceAudio, ImportWorkspaceAudioParams};
pub use move_entry::{MoveWorkspaceEntry, MoveWorkspaceEntryParams};
pub use new::{NewWorkspace, NewWorkspaceParams};
pub use open::{OpenWorkspace, OpenWorkspaceParams};
pub use save::{SaveWorkspace, SaveWorkspaceParams};
pub use save_as::{SaveWorkspaceAs, SaveWorkspaceAsParams};
pub use save_timeline_clip::{SaveTimelineClip, SaveTimelineClipParams};
pub use save_timeline_track::{SaveTimelineTrack, SaveTimelineTrackParams};
pub use set_timeline_settings::{SetTimelineSettings, SetTimelineSettingsParams};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ContractMethod, contract::describe};

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub name: String,
    pub root_path: Option<String>,
    pub project_file_path: Option<String>,
    pub files: Vec<String>,
    #[ts(inline)]
    pub timeline: TimelineSnapshot,
    pub is_dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct TimelineSnapshot {
    pub ticks_per_quarter: u32,
    pub bpm: f64,
    pub time_signature_numerator: u8,
    pub time_signature_denominator: u8,
    #[ts(inline)]
    pub grid_division: GridDivision,
    pub is_snap_enabled: bool,
    #[ts(inline)]
    pub tracks: Vec<TimelineTrack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct TimelineTrack {
    pub id: String,
    pub name: String,
    pub is_muted: bool,
    pub is_soloed: bool,
    #[ts(inline)]
    pub clips: Vec<TimelineClip>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct TimelineClip {
    pub id: String,
    pub name: String,
    pub start_tick: u32,
    pub duration_ticks: u32,
    pub source_offset_ticks: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum GridDivision {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    QuarterTriplet,
    EighthTriplet,
    SixteenthTriplet,
}

pub(crate) fn methods() -> Vec<ContractMethod> {
    vec![
        describe::<CreateWorkspaceDirectory>(),
        describe::<DeleteWorkspaceEntry>(),
        describe::<DeleteTimelineClip>(),
        describe::<DeleteTimelineTrack>(),
        describe::<GetWorkspace>(),
        describe::<ImportWorkspaceAudio>(),
        describe::<MoveWorkspaceEntry>(),
        describe::<NewWorkspace>(),
        describe::<OpenWorkspace>(),
        describe::<SaveWorkspace>(),
        describe::<SaveWorkspaceAs>(),
        describe::<SaveTimelineClip>(),
        describe::<SaveTimelineTrack>(),
        describe::<SetTimelineSettings>(),
    ]
}
