use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{ErrorCode, workspace::SetTimelineSettingsParams};
use serde_json::Value;

use super::{ApiError, core_error, deserialize_grid_division, serialize_snapshot};

pub(super) fn handle(params: &Value, workspaces: &mut Workspaces) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<SetTimelineSettingsParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    workspaces
        .set_timeline_settings(
            params.bpm,
            params.time_signature_numerator,
            params.time_signature_denominator,
            deserialize_grid_division(params.grid_division),
            params.is_snap_enabled,
        )
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
