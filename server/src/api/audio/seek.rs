use kickhatsnare_core::Core;
use kickhatsnare_protocol::{ErrorCode, audio::SeekAudioParams};
use serde_json::Value;

use super::{ApiError, serialize_transport};

pub(super) fn handle(params: &Value, core: &mut Core) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<SeekAudioParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    serialize_transport(core.seek_audio(params.position_tick))
}
