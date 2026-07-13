use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{ErrorCode, workspace::SaveTimelineTrackParams};
use serde_json::Value;

use super::{ApiError, core_error, serialize_snapshot};

pub(super) fn handle(params: &Value, workspaces: &mut Workspaces) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<SaveTimelineTrackParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    workspaces
        .save_timeline_track(
            params.id.as_deref(),
            &params.name,
            params.is_muted,
            params.is_soloed,
        )
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
