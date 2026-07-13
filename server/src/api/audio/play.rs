use kickhatsnare_core::Core;
use kickhatsnare_protocol::{ErrorCode, audio::PlayAudioParams};
use serde_json::Value;

use super::{ApiError, serialize_transport};

pub(super) fn handle(params: &Value, core: &mut Core) -> Result<Value, ApiError> {
    serde_json::from_value::<PlayAudioParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    core.play_audio()
        .map_err(|error| ApiError::new(ErrorCode::InternalError, error.to_string()))
        .and_then(serialize_transport)
}
