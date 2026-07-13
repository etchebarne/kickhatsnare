use kickhatsnare_core::library::Library;
use kickhatsnare_protocol::{ErrorCode, library::PinFolderParams};
use serde_json::Value;

use super::{ApiError, core_error, serialize_snapshot};

pub(super) fn handle(params: &Value, library: &mut Library) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<PinFolderParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    library
        .pin_folder(params.path)
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
