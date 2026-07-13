use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct NewWorkspace;

impl IpcMethod for NewWorkspace {
    const NAME: &'static str = "workspace.new";
    type Params = NewWorkspaceParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct NewWorkspaceParams {}
