use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct SetMasterMix;

impl IpcMethod for SetMasterMix {
    const NAME: &'static str = "workspace.setMasterMix";
    type Params = SetMasterMixParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SetMasterMixParams {
    pub gain_db: f64,
    pub is_muted: bool,
}
