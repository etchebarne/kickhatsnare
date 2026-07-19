use std::path::PathBuf;

use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{ErrorCode, workspace::ReconcileMovedWorkspaceFilesParams};
use serde_json::Value;

use super::{ApiError, core_error, serialize_snapshot};

pub(super) fn handle(params: &Value, workspaces: &mut Workspaces) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<ReconcileMovedWorkspaceFilesParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    let moves = params.moves.into_iter().map(|entry| {
        (
            PathBuf::from(entry.source_path),
            PathBuf::from(entry.destination_path),
        )
    });
    workspaces
        .reconcile_moved_files(moves)
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
