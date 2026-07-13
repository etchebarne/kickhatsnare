use kickhatsnare_core::{Core, settings::SettingValue as CoreSettingValue};
use kickhatsnare_protocol::{
    ErrorCode,
    settings::{SetSettingParams, SettingValue},
};
use serde_json::Value;

use super::{ApiError, serialize_snapshot};

pub(super) fn handle(params: &Value, core: &mut Core) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<SetSettingParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    let value = match params.value {
        SettingValue::Integer(value) => CoreSettingValue::Integer(value),
    };
    core.set_setting(&params.id, value)
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))
        .and_then(serialize_snapshot)
}
