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

use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{
    IpcMethod,
    workspace::{
        CreateWorkspaceDirectory, DeleteTimelineClip, DeleteTimelineTrack, DeleteWorkspaceEntry,
        GetWorkspace, ImportWorkspaceAudio, MoveWorkspaceEntry, NewWorkspace, OpenWorkspace,
        SaveTimelineClip, SaveTimelineTrack, SaveWorkspace, SaveWorkspaceAs, SetTimelineSettings,
    },
};
use serde_json::Value;

use super::ApiError;

pub(super) fn dispatch(
    method: &str,
    action: &str,
    params: &Value,
    workspaces: &mut Workspaces,
) -> Result<Value, ApiError> {
    match method {
        CreateWorkspaceDirectory::NAME => create_directory::handle(params, workspaces),
        DeleteWorkspaceEntry::NAME => delete_entry::handle(params, workspaces),
        DeleteTimelineClip::NAME => delete_timeline_clip::handle(params, workspaces),
        DeleteTimelineTrack::NAME => delete_timeline_track::handle(params, workspaces),
        GetWorkspace::NAME => get::handle(params, workspaces),
        ImportWorkspaceAudio::NAME => import_audio::handle(params, workspaces),
        MoveWorkspaceEntry::NAME => move_entry::handle(params, workspaces),
        NewWorkspace::NAME => new::handle(params, workspaces),
        OpenWorkspace::NAME => open::handle(params, workspaces),
        SaveWorkspace::NAME => save::handle(params, workspaces),
        SaveWorkspaceAs::NAME => save_as::handle(params, workspaces),
        SaveTimelineClip::NAME => save_timeline_clip::handle(params, workspaces),
        SaveTimelineTrack::NAME => save_timeline_track::handle(params, workspaces),
        SetTimelineSettings::NAME => set_timeline_settings::handle(params, workspaces),
        _ => Err(ApiError::method_not_found("workspace", action)),
    }
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
            tracks: snapshot
                .timeline
                .tracks
                .into_iter()
                .map(|track| kickhatsnare_protocol::workspace::TimelineTrack {
                    id: track.id,
                    name: track.name,
                    is_muted: track.is_muted,
                    is_soloed: track.is_soloed,
                    clips: track
                        .clips
                        .into_iter()
                        .map(|clip| kickhatsnare_protocol::workspace::TimelineClip {
                            id: clip.id,
                            name: clip.name,
                            start_tick: clip.start_tick,
                            duration_ticks: clip.duration_ticks,
                            source_offset_ticks: clip.source_offset_ticks,
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
