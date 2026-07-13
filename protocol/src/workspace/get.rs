use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct GetWorkspace;

impl IpcMethod for GetWorkspace {
    const NAME: &'static str = "workspace.get";
    type Params = GetWorkspaceParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct GetWorkspaceParams {}
