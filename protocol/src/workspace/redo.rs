use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct RedoWorkspace;

impl IpcMethod for RedoWorkspace {
    const NAME: &'static str = "workspace.redo";
    type Params = RedoWorkspaceParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct RedoWorkspaceParams {}
