use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{ErrorCode, workspace::UndoWorkspaceParams};
use serde_json::Value;

use super::{ApiError, core_error, serialize_snapshot};

pub(super) fn handle(params: &Value, workspaces: &mut Workspaces) -> Result<Value, ApiError> {
    serde_json::from_value::<UndoWorkspaceParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    workspaces
        .undo()
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
