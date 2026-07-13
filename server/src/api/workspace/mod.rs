mod create_directory;
mod delete_entry;
mod get;
mod import_audio;
mod move_entry;
mod new;
mod open;
mod save;
mod save_as;

use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{
    IpcMethod,
    workspace::{
        CreateWorkspaceDirectory, DeleteWorkspaceEntry, GetWorkspace, ImportWorkspaceAudio,
        MoveWorkspaceEntry, NewWorkspace, OpenWorkspace, SaveWorkspace, SaveWorkspaceAs,
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
        GetWorkspace::NAME => get::handle(params, workspaces),
        ImportWorkspaceAudio::NAME => import_audio::handle(params, workspaces),
        MoveWorkspaceEntry::NAME => move_entry::handle(params, workspaces),
        NewWorkspace::NAME => new::handle(params, workspaces),
        OpenWorkspace::NAME => open::handle(params, workspaces),
        SaveWorkspace::NAME => save::handle(params, workspaces),
        SaveWorkspaceAs::NAME => save_as::handle(params, workspaces),
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
        is_dirty: snapshot.is_dirty,
    };

    serde_json::to_value(snapshot).map_err(|error| {
        ApiError::new(
            kickhatsnare_protocol::ErrorCode::InternalError,
            error.to_string(),
        )
    })
}

fn core_error(error: &kickhatsnare_core::CoreError) -> ApiError {
    ApiError::new(
        kickhatsnare_protocol::ErrorCode::InternalError,
        error.to_string(),
    )
}
