use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{ErrorCode, workspace::SaveTimelineClipParams};
use serde_json::Value;

use super::{ApiError, core_error, deserialize_clip_resize_mode, serialize_snapshot};

pub(super) fn handle(params: &Value, workspaces: &mut Workspaces) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<SaveTimelineClipParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    workspaces
        .save_timeline_clip(
            params.id.as_deref(),
            &params.track_id,
            &params.name,
            params.start_tick,
            params.duration_ticks,
            params.source_offset_ticks,
            params.source_duration_ticks,
            deserialize_clip_resize_mode(params.resize_mode),
        )
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
