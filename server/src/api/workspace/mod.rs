use kickhatsnare_core::workspace::Workspaces;
use serde_json::Value;

use super::ApiError;

pub(super) fn dispatch(
    _method: &str,
    action: &str,
    _params: &Value,
    _workspaces: &mut Workspaces,
) -> Result<Value, ApiError> {
    Err(ApiError::method_not_found("workspace", action))
}
