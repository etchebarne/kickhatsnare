use kickhatsnare_core::{system::PingReply, system::System};
use kickhatsnare_protocol::{
    ErrorCode,
    system::{PingParams, PingResult},
};
use serde_json::Value;

use super::ApiError;

pub(super) fn handle(params: &Value, system: &mut System) -> Result<Value, ApiError> {
    serde_json::from_value::<PingParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;

    let result = match system.ping() {
        PingReply::Pong => PingResult::Ready,
    };

    serde_json::to_value(result)
        .map_err(|error| ApiError::new(ErrorCode::InternalError, error.to_string()))
}
