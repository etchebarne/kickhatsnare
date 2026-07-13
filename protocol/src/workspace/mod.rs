mod add_audio_clip;
mod connect_mix_ports;
mod create_directory;
mod delete_entry;
mod delete_timeline_clip;
mod delete_timeline_track;
mod disconnect_mix_ports;
mod get;
mod import_audio;
mod move_entry;
mod new;
mod open;
mod redo;
mod save;
mod save_as;
mod save_timeline_clip;
mod save_timeline_track;
mod set_master_mix;
mod set_mix_node_position;
mod set_timeline_settings;
mod undo;

pub use add_audio_clip::{AddAudioClip, AddAudioClipParams};
pub use connect_mix_ports::{ConnectMixPorts, ConnectMixPortsParams};
pub use create_directory::{CreateWorkspaceDirectory, CreateWorkspaceDirectoryParams};
pub use delete_entry::{DeleteWorkspaceEntry, DeleteWorkspaceEntryParams};
pub use delete_timeline_clip::{DeleteTimelineClip, DeleteTimelineClipParams};
pub use delete_timeline_track::{DeleteTimelineTrack, DeleteTimelineTrackParams};
pub use disconnect_mix_ports::{DisconnectMixPorts, DisconnectMixPortsParams};
pub use get::{GetWorkspace, GetWorkspaceParams};
pub use import_audio::{ImportWorkspaceAudio, ImportWorkspaceAudioParams};
pub use move_entry::{MoveWorkspaceEntry, MoveWorkspaceEntryParams};
pub use new::{NewWorkspace, NewWorkspaceParams};
pub use open::{OpenWorkspace, OpenWorkspaceParams};
pub use redo::{RedoWorkspace, RedoWorkspaceParams};
pub use save::{SaveWorkspace, SaveWorkspaceParams};
pub use save_as::{SaveWorkspaceAs, SaveWorkspaceAsParams};
pub use save_timeline_clip::{SaveTimelineClip, SaveTimelineClipParams};
pub use save_timeline_track::{SaveTimelineTrack, SaveTimelineTrackParams};
pub use set_master_mix::{SetMasterMix, SetMasterMixParams};
pub use set_mix_node_position::{SetMixNodePosition, SetMixNodePositionParams};
pub use set_timeline_settings::{SetTimelineSettings, SetTimelineSettingsParams};
pub use undo::{UndoWorkspace, UndoWorkspaceParams};

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
    #[ts(inline)]
    pub history: WorkspaceHistorySnapshot,
    pub is_dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct WorkspaceHistorySnapshot {
    pub can_undo: bool,
    pub can_redo: bool,
    pub undo_label: Option<String>,
    pub redo_label: Option<String>,
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
    pub master_gain_db: f64,
    pub is_master_muted: bool,
    #[ts(inline)]
    pub mix_graph: MixGraph,
    #[ts(inline)]
    pub tracks: Vec<TimelineTrack>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct TimelineTrack {
    pub id: String,
    pub name: String,
    pub is_muted: bool,
    pub is_soloed: bool,
    pub gain_db: f64,
    pub pan: f64,
    #[ts(inline)]
    pub clips: Vec<TimelineClip>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MixGraph {
    #[ts(inline)]
    pub nodes: Vec<MixNode>,
    #[ts(inline)]
    pub connections: Vec<MixConnection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MixNode {
    pub id: String,
    #[ts(inline)]
    pub kind: MixNodeKind,
    pub track_id: Option<String>,
    pub x: f64,
    pub y: f64,
    #[ts(inline)]
    pub ports: Vec<MixPort>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MixPort {
    pub id: String,
    pub label: String,
    #[ts(inline)]
    pub direction: MixPortDirection,
    #[ts(inline)]
    pub signal_type: MixSignalType,
    pub allows_multiple_connections: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MixConnection {
    pub source_node_id: String,
    pub source_port_id: String,
    pub target_node_id: String,
    pub target_port_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum MixNodeKind {
    TrackChannel,
    MasterOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum MixPortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum MixSignalType {
    Audio,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct TimelineClip {
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
        describe::<AddAudioClip>(),
        describe::<ConnectMixPorts>(),
        describe::<CreateWorkspaceDirectory>(),
        describe::<DeleteWorkspaceEntry>(),
        describe::<DeleteTimelineClip>(),
        describe::<DeleteTimelineTrack>(),
        describe::<DisconnectMixPorts>(),
        describe::<GetWorkspace>(),
        describe::<ImportWorkspaceAudio>(),
        describe::<MoveWorkspaceEntry>(),
        describe::<NewWorkspace>(),
        describe::<OpenWorkspace>(),
        describe::<RedoWorkspace>(),
        describe::<SaveWorkspace>(),
        describe::<SaveWorkspaceAs>(),
        describe::<SaveTimelineClip>(),
        describe::<SaveTimelineTrack>(),
        describe::<SetTimelineSettings>(),
        describe::<SetMasterMix>(),
        describe::<SetMixNodePosition>(),
        describe::<UndoWorkspace>(),
    ]
}
