use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{ErrorCode, workspace::SetTimelineClipPropertiesParams};
use serde_json::Value;

use super::{ApiError, core_error, deserialize_clip_stretch_mode, serialize_snapshot};

pub(super) fn handle(params: &Value, workspaces: &mut Workspaces) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<SetTimelineClipPropertiesParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    workspaces
        .set_timeline_clip_properties(
            &params.id,
            deserialize_clip_stretch_mode(params.stretch_mode),
            params.gain_db,
            params.pan,
            params.pitch_semitones,
            params.tempo_percent,
            params.make_unique,
        )
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
