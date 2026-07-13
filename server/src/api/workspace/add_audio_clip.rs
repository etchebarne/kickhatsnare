use kickhatsnare_core::Core;
use kickhatsnare_protocol::{ErrorCode, workspace::AddAudioClipParams};
use serde_json::Value;

use super::{ApiError, core_error, serialize_snapshot};

pub(super) fn handle(params: &Value, core: &mut Core) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<AddAudioClipParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    core.add_audio_clip(&params.track_id, &params.source_path, params.start_tick)
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
