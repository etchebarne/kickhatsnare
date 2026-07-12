use kickhatsnare_core::audio::Audio;
use serde_json::Value;

use super::ApiError;

pub(super) fn dispatch(
    _method: &str,
    action: &str,
    _params: &Value,
    _audio: &mut Audio,
) -> Result<Value, ApiError> {
    Err(ApiError::method_not_found("audio", action))
}
