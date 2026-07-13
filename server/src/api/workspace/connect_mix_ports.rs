use kickhatsnare_core::workspace::Workspaces;
use kickhatsnare_protocol::{ErrorCode, workspace::ConnectMixPortsParams};
use serde_json::Value;

use super::{ApiError, core_error, serialize_snapshot};

pub(super) fn handle(params: &Value, workspaces: &mut Workspaces) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<ConnectMixPortsParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    workspaces
        .connect_mix_ports(
            &params.source_node_id,
            &params.source_port_id,
            &params.target_node_id,
            &params.target_port_id,
        )
        .map_err(|error| core_error(&error))
        .and_then(serialize_snapshot)
}
