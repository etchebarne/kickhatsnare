use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct SetMixNodePosition;

impl IpcMethod for SetMixNodePosition {
    const NAME: &'static str = "workspace.setMixNodePosition";
    type Params = SetMixNodePositionParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SetMixNodePositionParams {
    pub node_id: String,
    pub x: f64,
    pub y: f64,
}
