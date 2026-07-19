use kickhatsnare_core::{Core, workspace::MissingMediaRecovery};
use kickhatsnare_protocol::{
    ErrorCode,
    workspace::{MissingMediaAction, RecoverMissingWorkspaceMediaParams},
};
use serde_json::Value;

use super::{ApiError, core_error, serialize_snapshot};

pub(super) fn handle(params: &Value, core: &mut Core) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<RecoverMissingWorkspaceMediaParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    let snapshot = match (params.action, params.replacement_path) {
        (MissingMediaAction::Replace, Some(replacement_path)) => {
            core.replace_missing_media(&params.source_path, &replacement_path)
        }
        (MissingMediaAction::Replace, None) => {
            return Err(ApiError::new(
                ErrorCode::InvalidParams,
                "replacementPath is required when replacing missing media",
            ));
        }
        (MissingMediaAction::LeaveEmpty, None) => core
            .workspaces()
            .recover_missing_media(&params.source_path, MissingMediaRecovery::LeaveEmpty),
        (MissingMediaAction::DeleteClips, None) => core
            .workspaces()
            .recover_missing_media(&params.source_path, MissingMediaRecovery::DeleteClips),
        (_, Some(_)) => {
            return Err(ApiError::new(
                ErrorCode::InvalidParams,
                "replacementPath is only valid when replacing missing media",
            ));
        }
    };
    snapshot
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
