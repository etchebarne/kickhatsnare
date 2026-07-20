use kickhatsnare_core::{Core, audio::LoopRegion};
use kickhatsnare_protocol::{ErrorCode, audio::SetLoopRegionParams};
use serde_json::Value;

use super::{ApiError, serialize_transport};

pub(super) fn handle(params: &Value, core: &mut Core) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<SetLoopRegionParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    let region = params.region.map(|region| LoopRegion {
        start_tick: region.start_tick,
        end_tick: region.end_tick,
    });
    let snapshot = core
        .set_audio_loop_region(region)
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    serialize_transport(snapshot)
}
