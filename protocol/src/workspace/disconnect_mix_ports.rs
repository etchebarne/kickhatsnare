use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct DisconnectMixPorts;

impl IpcMethod for DisconnectMixPorts {
    const NAME: &'static str = "workspace.disconnectMixPorts";
    type Params = DisconnectMixPortsParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct DisconnectMixPortsParams {
    pub source_node_id: String,
    pub source_port_id: String,
    pub target_node_id: String,
    pub target_port_id: String,
}
