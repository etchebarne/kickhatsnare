mod add_audio_clip;
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
mod set_master_mix;
mod set_mix_node_position;
mod set_timeline_settings;

use kickhatsnare_core::Core;
use kickhatsnare_protocol::{
    IpcMethod,
    workspace::{
        AddAudioClip, CreateWorkspaceDirectory, DeleteTimelineClip, DeleteTimelineTrack,
        DeleteWorkspaceEntry, GetWorkspace, ImportWorkspaceAudio, MoveWorkspaceEntry, NewWorkspace,
        OpenWorkspace, SaveTimelineClip, SaveTimelineTrack, SaveWorkspace, SaveWorkspaceAs,
        SetMasterMix, SetMixNodePosition, SetTimelineSettings,
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
        CreateWorkspaceDirectory::NAME => create_directory::handle(params, core.workspaces()),
        DeleteWorkspaceEntry::NAME => delete_entry::handle(params, core.workspaces()),
        DeleteTimelineClip::NAME => delete_timeline_clip::handle(params, core.workspaces()),
        DeleteTimelineTrack::NAME => delete_timeline_track::handle(params, core.workspaces()),
        GetWorkspace::NAME => get::handle(params, core.workspaces()),
        ImportWorkspaceAudio::NAME => import_audio::handle(params, core.workspaces()),
        MoveWorkspaceEntry::NAME => move_entry::handle(params, core.workspaces()),
        NewWorkspace::NAME => new::handle(params, core.workspaces()),
        OpenWorkspace::NAME => open::handle(params, core.workspaces()),
        SaveWorkspace::NAME => save::handle(params, core.workspaces()),
        SaveWorkspaceAs::NAME => save_as::handle(params, core.workspaces()),
        SaveTimelineClip::NAME => save_timeline_clip::handle(params, core.workspaces()),
        SaveTimelineTrack::NAME => save_timeline_track::handle(params, core.workspaces()),
        SetMasterMix::NAME => set_master_mix::handle(params, core.workspaces()),
        SetMixNodePosition::NAME => set_mix_node_position::handle(params, core.workspaces()),
        SetTimelineSettings::NAME => set_timeline_settings::handle(params, core.workspaces()),
        _ => Err(ApiError::method_not_found("workspace", action)),
    };
    if result.is_ok() {
        match method {
            SaveTimelineTrack::NAME | SetMasterMix::NAME => {
                core.sync_audio_mix().map_err(|error| core_error(&error))?;
            }
            AddAudioClip::NAME
            | DeleteTimelineClip::NAME
            | DeleteTimelineTrack::NAME
            | NewWorkspace::NAME
            | OpenWorkspace::NAME
            | SaveTimelineClip::NAME
            | SetTimelineSettings::NAME => core.invalidate_audio(),
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
            master_node_x: snapshot.timeline.master_node_x,
            master_node_y: snapshot.timeline.master_node_y,
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
                    is_connected: track.is_connected,
                    node_x: track.node_x,
                    node_y: track.node_y,
                    clips: track
                        .clips
                        .into_iter()
                        .map(|clip| kickhatsnare_protocol::workspace::TimelineClip {
                            id: clip.id,
                            name: clip.name,
                            start_tick: clip.start_tick,
                            duration_ticks: clip.duration_ticks,
                            source_offset_ticks: clip.source_offset_ticks,
                            source_path: clip.source_path,
                            source_sample_rate: clip.source_sample_rate,
                            source_channels: clip.source_channels,
                            source_duration_seconds: clip.source_duration_seconds,
                            waveform: clip.waveform,
                        })
                        .collect(),
                })
                .collect(),
        },
        is_dirty: snapshot.is_dirty,
    };

    serde_json::to_value(snapshot).map_err(|error| {
        ApiError::new(
            kickhatsnare_protocol::ErrorCode::InternalError,
            error.to_string(),
        )
    })
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

fn core_error(error: &kickhatsnare_core::CoreError) -> ApiError {
    ApiError::new(
        kickhatsnare_protocol::ErrorCode::InternalError,
        error.to_string(),
    )
}
