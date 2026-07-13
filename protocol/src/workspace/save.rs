use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct SaveWorkspace;

impl IpcMethod for SaveWorkspace {
    const NAME: &'static str = "workspace.save";
    type Params = SaveWorkspaceParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct SaveWorkspaceParams {}
