use std::path::PathBuf;

use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{ErrorCode, workspace::ImportWorkspaceAudioParams};
use serde_json::Value;

use super::{ApiError, core_error, serialize_snapshot};

pub(super) fn handle(params: &Value, workspaces: &mut Workspaces) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<ImportWorkspaceAudioParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    workspaces
        .import_audio_files(
            params.source_paths.into_iter().map(PathBuf::from),
            params.target_directory,
        )
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
