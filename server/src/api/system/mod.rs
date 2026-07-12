mod ping;

use kickhatsnare_core::system::System;
use kickhatsnare_protocol::{IpcMethod, system::Ping};
use serde_json::Value;

use super::ApiError;

pub(super) fn dispatch(
    method: &str,
    action: &str,
    params: &Value,
    system: &mut System,
) -> Result<Value, ApiError> {
    match method {
        Ping::NAME => ping::handle(params, system),
        _ => Err(ApiError::method_not_found("system", action)),
    }
}
