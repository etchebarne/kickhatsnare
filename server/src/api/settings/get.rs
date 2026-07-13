use kickhatsnare_core::Core;
use kickhatsnare_protocol::{ErrorCode, settings::GetSettingsParams};
use serde_json::Value;

use super::{ApiError, core_error, serialize_snapshot};

pub(super) fn handle(params: &Value, core: &Core) -> Result<Value, ApiError> {
    serde_json::from_value::<GetSettingsParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    core.settings_snapshot()
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
