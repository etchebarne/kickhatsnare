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
mod set_timeline_clip_properties;
mod set_timeline_settings;
mod split_timeline_clip;
mod undo;

use kickhatsnare_core::{Core, workspace::WorkspaceEditImpact};
use kickhatsnare_protocol::{
    IpcMethod,
    workspace::{
        AddAudioClip, ConnectMixPorts, CreateWorkspaceDirectory, DeleteTimelineClip,
        DeleteTimelineTrack, DeleteWorkspaceEntry, DisconnectMixPorts, GetWorkspace,
        ImportWorkspaceAudio, MoveWorkspaceEntry, NewWorkspace, OpenWorkspace, RedoWorkspace,
        SaveTimelineClip, SaveTimelineTrack, SaveWorkspace, SaveWorkspaceAs, SetMasterMix,
        SetMixNodePosition, SetTimelineClipProperties, SetTimelineSettings, SplitTimelineClip,
        UndoWorkspace,
    },
};
use serde_json::Value;

use super::ApiError;

pub(super) fn dispatch(
    method: &str,
    action: &str,
    params: &Value,
    core: &mut Core,
) -> Result<Value, ApiError> {
    let result = match method {
        AddAudioClip::NAME => add_audio_clip::handle(params, core),
        ConnectMixPorts::NAME => connect_mix_ports::handle(params, core.workspaces()),
        CreateWorkspaceDirectory::NAME => create_directory::handle(params, core.workspaces()),
        DeleteWorkspaceEntry::NAME => delete_entry::handle(params, core.workspaces()),
        DeleteTimelineClip::NAME => delete_timeline_clip::handle(params, core.workspaces()),
        DeleteTimelineTrack::NAME => delete_timeline_track::handle(params, core.workspaces()),
        DisconnectMixPorts::NAME => disconnect_mix_ports::handle(params, core.workspaces()),
        GetWorkspace::NAME => get::handle(params, core.workspaces()),
        ImportWorkspaceAudio::NAME => import_audio::handle(params, core.workspaces()),
        MoveWorkspaceEntry::NAME => move_entry::handle(params, core.workspaces()),
        NewWorkspace::NAME => new::handle(params, core.workspaces()),
        OpenWorkspace::NAME => open::handle(params, core.workspaces()),
        RedoWorkspace::NAME => redo::handle(params, core.workspaces()),
        SaveWorkspace::NAME => save::handle(params, core.workspaces()),
        SaveWorkspaceAs::NAME => save_as::handle(params, core.workspaces()),
        SaveTimelineClip::NAME => save_timeline_clip::handle(params, core.workspaces()),
        SaveTimelineTrack::NAME => save_timeline_track::handle(params, core.workspaces()),
        SetMasterMix::NAME => set_master_mix::handle(params, core.workspaces()),
        SetMixNodePosition::NAME => set_mix_node_position::handle(params, core.workspaces()),
        SetTimelineClipProperties::NAME => {
            set_timeline_clip_properties::handle(params, core.workspaces())
        }
        SetTimelineSettings::NAME => set_timeline_settings::handle(params, core.workspaces()),
        SplitTimelineClip::NAME => split_timeline_clip::handle(params, core.workspaces()),
        UndoWorkspace::NAME => undo::handle(params, core.workspaces()),
        _ => Err(ApiError::method_not_found("workspace", action)),
    };
    if result.is_ok() {
        match method {
            ConnectMixPorts::NAME | DisconnectMixPorts::NAME | SetMasterMix::NAME => {
                core.sync_audio_mix().map_err(|error| core_error(&error))?;
            }
            DeleteTimelineClip::NAME
            | DeleteTimelineTrack::NAME
            | SaveTimelineClip::NAME
            | SetTimelineClipProperties::NAME
            | SplitTimelineClip::NAME => {
                core.refresh_audio_timeline()
                    .map_err(|error| core_error(&error))?;
            }
            NewWorkspace::NAME | OpenWorkspace::NAME => {
                core.invalidate_audio();
            }
            RedoWorkspace::NAME
            | SaveTimelineTrack::NAME
            | SetTimelineSettings::NAME
            | UndoWorkspace::NAME => match core.workspaces().latest_history_impact() {
                WorkspaceEditImpact::None => {}
                WorkspaceEditImpact::Mix => {
                    core.sync_audio_mix().map_err(|error| core_error(&error))?;
                }
                WorkspaceEditImpact::Timeline => {
                    core.refresh_audio_timeline()
                        .map_err(|error| core_error(&error))?;
                }
            },
            _ => {}
        }
    }
    result
}

fn serialize_snapshot(
    snapshot: kickhatsnare_core::workspace::WorkspaceSnapshot,
) -> Result<Value, ApiError> {
    let snapshot = kickhatsnare_protocol::workspace::WorkspaceSnapshot {
        name: snapshot.name,
        root_path: snapshot
            .root_path
            .map(|path| path.to_string_lossy().into_owned()),
        project_file_path: snapshot
            .project_file_path
            .map(|path| path.to_string_lossy().into_owned()),
        files: snapshot.files,
        timeline: kickhatsnare_protocol::workspace::TimelineSnapshot {
            ticks_per_quarter: snapshot.timeline.ticks_per_quarter,
            bpm: snapshot.timeline.bpm,
            time_signature_numerator: snapshot.timeline.time_signature_numerator,
            time_signature_denominator: snapshot.timeline.time_signature_denominator,
            grid_division: serialize_grid_division(snapshot.timeline.grid_division),
            is_snap_enabled: snapshot.timeline.is_snap_enabled,
            master_gain_db: snapshot.timeline.master_gain_db,
            is_master_muted: snapshot.timeline.is_master_muted,
            mix_graph: kickhatsnare_protocol::workspace::MixGraph {
                nodes: snapshot
                    .timeline
                    .mix_graph
                    .nodes
                    .into_iter()
                    .map(|node| kickhatsnare_protocol::workspace::MixNode {
                        id: node.id,
                        kind: serialize_mix_node_kind(node.kind),
                        track_id: node.track_id,
                        x: node.x,
                        y: node.y,
                        ports: node
                            .ports
                            .into_iter()
                            .map(|port| kickhatsnare_protocol::workspace::MixPort {
                                id: port.id,
                                label: port.label,
                                direction: serialize_mix_port_direction(port.direction),
                                signal_type: serialize_mix_signal_type(port.signal_type),
                                allows_multiple_connections: port.allows_multiple_connections,
                            })
                            .collect(),
                    })
                    .collect(),
                connections: snapshot
                    .timeline
                    .mix_graph
                    .connections
                    .into_iter()
                    .map(
                        |connection| kickhatsnare_protocol::workspace::MixConnection {
                            source_node_id: connection.source_node_id,
                            source_port_id: connection.source_port_id,
                            target_node_id: connection.target_node_id,
                            target_port_id: connection.target_port_id,
                        },
                    )
                    .collect(),
            },
            tracks: snapshot
                .timeline
                .tracks
                .into_iter()
                .map(|track| kickhatsnare_protocol::workspace::TimelineTrack {
                    id: track.id,
                    name: track.name,
                    is_muted: track.is_muted,
                    is_soloed: track.is_soloed,
                    gain_db: track.gain_db,
                    pan: track.pan,
                    clips: track.clips.into_iter().map(serialize_clip).collect(),
                })
                .collect(),
        },
        history: serialize_history(snapshot.history),
        is_dirty: snapshot.is_dirty,
    };

    serde_json::to_value(snapshot).map_err(|error| {
        ApiError::new(
            kickhatsnare_protocol::ErrorCode::InternalError,
            error.to_string(),
        )
    })
}

fn serialize_clip(
    clip: kickhatsnare_core::workspace::TimelineClipSnapshot,
) -> kickhatsnare_protocol::workspace::TimelineClip {
    kickhatsnare_protocol::workspace::TimelineClip {
        id: clip.id,
        name: clip.name,
        start_tick: clip.start_tick,
        duration_ticks: clip.duration_ticks,
        source_offset_ticks: clip.source_offset_ticks,
        source_duration_ticks: clip.source_duration_ticks,
        source_path: clip.source_path,
        source_sample_rate: clip.source_sample_rate,
        source_channels: clip.source_channels,
        source_duration_seconds: clip.source_duration_seconds,
        waveform: clip.waveform,
        stretch_mode: serialize_clip_stretch_mode(clip.stretch_mode),
        gain_db: clip.gain_db,
        pan: clip.pan,
        pitch_semitones: clip.pitch_semitones,
        tempo_percent: clip.tempo_percent,
        is_unique: clip.is_unique,
    }
}

fn serialize_history(
    history: kickhatsnare_core::workspace::WorkspaceHistorySnapshot,
) -> kickhatsnare_protocol::workspace::WorkspaceHistorySnapshot {
    kickhatsnare_protocol::workspace::WorkspaceHistorySnapshot {
        can_undo: history.can_undo,
        can_redo: history.can_redo,
        undo_label: history.undo_label,
        redo_label: history.redo_label,
    }
}

fn deserialize_grid_division(
    division: kickhatsnare_protocol::workspace::GridDivision,
) -> kickhatsnare_core::workspace::GridDivision {
    use kickhatsnare_protocol::workspace::GridDivision as Protocol;

    match division {
        Protocol::Whole => kickhatsnare_core::workspace::GridDivision::Whole,
        Protocol::Half => kickhatsnare_core::workspace::GridDivision::Half,
        Protocol::Quarter => kickhatsnare_core::workspace::GridDivision::Quarter,
        Protocol::Eighth => kickhatsnare_core::workspace::GridDivision::Eighth,
        Protocol::Sixteenth => kickhatsnare_core::workspace::GridDivision::Sixteenth,
        Protocol::ThirtySecond => kickhatsnare_core::workspace::GridDivision::ThirtySecond,
        Protocol::QuarterTriplet => kickhatsnare_core::workspace::GridDivision::QuarterTriplet,
        Protocol::EighthTriplet => kickhatsnare_core::workspace::GridDivision::EighthTriplet,
        Protocol::SixteenthTriplet => kickhatsnare_core::workspace::GridDivision::SixteenthTriplet,
    }
}

fn serialize_grid_division(
    division: kickhatsnare_core::workspace::GridDivision,
) -> kickhatsnare_protocol::workspace::GridDivision {
    use kickhatsnare_core::workspace::GridDivision as Core;

    match division {
        Core::Whole => kickhatsnare_protocol::workspace::GridDivision::Whole,
        Core::Half => kickhatsnare_protocol::workspace::GridDivision::Half,
        Core::Quarter => kickhatsnare_protocol::workspace::GridDivision::Quarter,
        Core::Eighth => kickhatsnare_protocol::workspace::GridDivision::Eighth,
        Core::Sixteenth => kickhatsnare_protocol::workspace::GridDivision::Sixteenth,
        Core::ThirtySecond => kickhatsnare_protocol::workspace::GridDivision::ThirtySecond,
        Core::QuarterTriplet => kickhatsnare_protocol::workspace::GridDivision::QuarterTriplet,
        Core::EighthTriplet => kickhatsnare_protocol::workspace::GridDivision::EighthTriplet,
        Core::SixteenthTriplet => kickhatsnare_protocol::workspace::GridDivision::SixteenthTriplet,
    }
}

pub(super) fn deserialize_clip_stretch_mode(
    mode: kickhatsnare_protocol::workspace::ClipStretchMode,
) -> kickhatsnare_core::workspace::ClipStretchMode {
    match mode {
        kickhatsnare_protocol::workspace::ClipStretchMode::Resample => {
            kickhatsnare_core::workspace::ClipStretchMode::Resample
        }
        kickhatsnare_protocol::workspace::ClipStretchMode::Stretch => {
            kickhatsnare_core::workspace::ClipStretchMode::Stretch
        }
    }
}

pub(super) fn deserialize_clip_resize_mode(
    mode: kickhatsnare_protocol::workspace::ClipResizeMode,
) -> kickhatsnare_core::workspace::ClipResizeMode {
    match mode {
        kickhatsnare_protocol::workspace::ClipResizeMode::Trim => {
            kickhatsnare_core::workspace::ClipResizeMode::Trim
        }
        kickhatsnare_protocol::workspace::ClipResizeMode::Stretch => {
            kickhatsnare_core::workspace::ClipResizeMode::Stretch
        }
    }
}

fn serialize_clip_stretch_mode(
    mode: kickhatsnare_core::workspace::ClipStretchMode,
) -> kickhatsnare_protocol::workspace::ClipStretchMode {
    match mode {
        kickhatsnare_core::workspace::ClipStretchMode::Resample => {
            kickhatsnare_protocol::workspace::ClipStretchMode::Resample
        }
        kickhatsnare_core::workspace::ClipStretchMode::Stretch => {
            kickhatsnare_protocol::workspace::ClipStretchMode::Stretch
        }
    }
}

fn serialize_mix_node_kind(
    kind: kickhatsnare_core::workspace::MixNodeKind,
) -> kickhatsnare_protocol::workspace::MixNodeKind {
    match kind {
        kickhatsnare_core::workspace::MixNodeKind::TrackChannel => {
            kickhatsnare_protocol::workspace::MixNodeKind::TrackChannel
        }
        kickhatsnare_core::workspace::MixNodeKind::MasterOutput => {
            kickhatsnare_protocol::workspace::MixNodeKind::MasterOutput
        }
    }
}

fn serialize_mix_port_direction(
    direction: kickhatsnare_core::workspace::MixPortDirection,
) -> kickhatsnare_protocol::workspace::MixPortDirection {
    match direction {
        kickhatsnare_core::workspace::MixPortDirection::Input => {
            kickhatsnare_protocol::workspace::MixPortDirection::Input
        }
        kickhatsnare_core::workspace::MixPortDirection::Output => {
            kickhatsnare_protocol::workspace::MixPortDirection::Output
        }
    }
}

fn serialize_mix_signal_type(
    signal_type: kickhatsnare_core::workspace::MixSignalType,
) -> kickhatsnare_protocol::workspace::MixSignalType {
    match signal_type {
        kickhatsnare_core::workspace::MixSignalType::Audio => {
            kickhatsnare_protocol::workspace::MixSignalType::Audio
        }
    }
}

fn core_error(error: &kickhatsnare_core::CoreError) -> ApiError {
    ApiError::new(
        kickhatsnare_protocol::ErrorCode::InternalError,
        error.to_string(),
    )
}
